import { parseCommandError, tauriInvoke } from '@/lib/tauri'
import type {
  ChatAskResponse,
  ChatMessage,
  ChatSession,
  ControlPlaneActions,
  GovernanceCopilotResponse,
} from '../types'
import type { ControlPlaneGet, ControlPlaneSet } from '../store-types'
import {
  DEFAULT_CHAT_SESSION_TITLE,
  MAX_CHAT_MESSAGES_PER_SESSION,
  MAX_CHAT_SESSIONS,
} from '../constants'
import {
  buildChatSession,
  deriveActiveChatMessages,
  deriveSessionTitleFromQuestion,
  formatChatErrorMessage,
  persistChatState,
  readStoredChatState,
} from '../helpers'

type ChatActionKeys =
  | 'chatAsk'
  | 'askGovernanceCopilot'
  | 'reportFeature'
  | 'clearChatMessages'
  | 'createChatSession'
  | 'setActiveChatSession'
  | 'closeChatSession'
  | 'refreshChatMessagesForActiveUser'

export function createChatActions(
  set: ControlPlaneSet,
  get: ControlPlaneGet,
): Pick<ControlPlaneActions, ChatActionKeys> {
  return {
  chatAsk: async (question, orgName) => {
    const { serverConfig, selectedOrgName } = get()
    if (!serverConfig) return null
    const effectiveOrgName = orgName?.trim() || selectedOrgName.trim() || undefined
    const questionTrimmed = question.trim()
    if (!questionTrimmed) return null

    let sessionId = get().activeChatSessionId
    if (!sessionId) {
      const seeded = buildChatSession()
      sessionId = seeded.id
      set((s) => ({
        chatSessions: [...s.chatSessions, seeded].slice(-MAX_CHAT_SESSIONS),
        activeChatSessionId: seeded.id,
        chatMessages: seeded.messages,
      }))
    }

    const userMsg: ChatMessage = {
      id: crypto.randomUUID(),
      role: 'user',
      content: questionTrimmed,
      timestamp: Date.now(),
    }

    set((s) => {
      const idx = s.chatSessions.findIndex((session) => session.id === sessionId)
      if (idx < 0) return { isChatLoading: true }
      const target = s.chatSessions[idx]
      const isFirstUserQuestion = !target.messages.some((m) => m.role === 'user')
      const nextSession: ChatSession = {
        ...target,
        title: isFirstUserQuestion ? deriveSessionTitleFromQuestion(questionTrimmed) : target.title,
        updated_at: Date.now(),
        messages: [...target.messages, userMsg].slice(-MAX_CHAT_MESSAGES_PER_SESSION),
      }
      const nextSessions = [...s.chatSessions]
      nextSessions[idx] = nextSession
      persistChatState(nextSessions, s.activeChatSessionId ?? nextSession.id)
      return {
        chatSessions: nextSessions,
        chatMessages: s.activeChatSessionId === nextSession.id ? nextSession.messages : s.chatMessages,
        isChatLoading: true,
      }
    })

    try {
      const response = await tauriInvoke<ChatAskResponse>('cmd_server_chat_ask', {
        config: serverConfig,
        request: { question: questionTrimmed, org_name: effectiveOrgName ?? null },
      })
      const assistantMsg: ChatMessage = {
        id: crypto.randomUUID(),
        role: 'assistant',
        content: response.answer,
        response,
        timestamp: Date.now(),
      }
      set((s) => {
        const idx = s.chatSessions.findIndex((session) => session.id === sessionId)
        if (idx < 0) return { isChatLoading: false }
        const target = s.chatSessions[idx]
        const nextSession: ChatSession = {
          ...target,
          updated_at: Date.now(),
          messages: [...target.messages, assistantMsg].slice(-MAX_CHAT_MESSAGES_PER_SESSION),
        }
        const nextSessions = [...s.chatSessions]
        nextSessions[idx] = nextSession
        persistChatState(nextSessions, s.activeChatSessionId ?? nextSession.id)
        return {
          chatSessions: nextSessions,
          chatMessages: s.activeChatSessionId === nextSession.id ? nextSession.messages : s.chatMessages,
          isChatLoading: false,
        }
      })
      return response
    } catch (e) {
      const parsedError = parseCommandError(String(e))
      const userFacingError = formatChatErrorMessage(parsedError.message)
      const errMsg: ChatMessage = {
        id: crypto.randomUUID(),
        role: 'assistant',
        content: `Error: ${userFacingError}`,
        response: { status: 'error', answer: userFacingError, can_report_feature: false, data_refs: [] },
        timestamp: Date.now(),
      }
      set((s) => {
        const idx = s.chatSessions.findIndex((session) => session.id === sessionId)
        if (idx < 0) return { isChatLoading: false }
        const target = s.chatSessions[idx]
        const nextSession: ChatSession = {
          ...target,
          updated_at: Date.now(),
          messages: [...target.messages, errMsg].slice(-MAX_CHAT_MESSAGES_PER_SESSION),
        }
        const nextSessions = [...s.chatSessions]
        nextSessions[idx] = nextSession
        persistChatState(nextSessions, s.activeChatSessionId ?? nextSession.id)
        return {
          chatSessions: nextSessions,
          chatMessages: s.activeChatSessionId === nextSession.id ? nextSession.messages : s.chatMessages,
          isChatLoading: false,
        }
      })
      return null
    }
  },

  askGovernanceCopilot: async (request) => {
    const { serverConfig, selectedOrgName } = get()
    const question = request.question.trim()
    if (!serverConfig) {
      set({ governanceCopilotError: 'Conecta el Control Plane antes de usar el copilot.' })
      return null
    }
    if (!question) {
      set({ governanceCopilotError: 'Escribe una pregunta para el copilot.' })
      return null
    }

    const effectiveOrgName = request.org_name?.trim() || selectedOrgName.trim() || undefined
    set({ isGovernanceCopilotLoading: true, governanceCopilotError: null })
    try {
      const response = await tauriInvoke<GovernanceCopilotResponse>('cmd_server_governance_copilot_ask', {
        config: serverConfig,
        request: {
          question,
          org_name: effectiveOrgName ?? null,
          repository_full_name: request.repository_full_name?.trim() || null,
          branch: request.branch?.trim() || null,
          ticket_id: request.ticket_id?.trim() || null,
          release_id: request.release_id?.trim() || null,
          environment: request.environment?.trim() || null,
          hours: request.hours ?? null,
        },
      })
      set({
        governanceCopilotResponse: response,
        isGovernanceCopilotLoading: false,
        governanceCopilotError: null,
      })
      return response
    } catch (e) {
      const parsedError = parseCommandError(String(e))
      set({
        governanceCopilotResponse: null,
        isGovernanceCopilotLoading: false,
        governanceCopilotError: parsedError.message,
      })
      return null
    }
  },

  reportFeature: async (question, missingCapability) => {
    const { serverConfig, userOrgId } = get()
    if (!serverConfig) return false
    try {
      await tauriInvoke<{ id: string; status: string }>('cmd_server_create_feature_request', {
        config: serverConfig,
        input: {
          question,
          missing_capability: missingCapability ?? null,
          org_id: userOrgId ?? null,
          user_login: null,
          metadata: null,
        },
      })
      return true
    } catch {
      return false
    }
  },

  clearChatMessages: () => {
    set((s) => {
      const activeId = s.activeChatSessionId
      if (!activeId) return {}
      const idx = s.chatSessions.findIndex((session) => session.id === activeId)
      if (idx < 0) return {}
      const target = s.chatSessions[idx]
      const nextSession: ChatSession = { ...target, messages: [], updated_at: Date.now(), title: target.title || DEFAULT_CHAT_SESSION_TITLE }
      const nextSessions = [...s.chatSessions]
      nextSessions[idx] = nextSession
      persistChatState(nextSessions, activeId)
      return { chatSessions: nextSessions, chatMessages: [] }
    })
  },

  createChatSession: () => {
    if (get().isChatLoading) return
    set((s) => {
      let nextSessions = [...s.chatSessions]
      if (nextSessions.length >= MAX_CHAT_SESSIONS) {
        const removableIdx = nextSessions.findIndex((session) => session.id !== s.activeChatSessionId)
        nextSessions.splice(removableIdx >= 0 ? removableIdx : 0, 1)
      }
      const newSession = buildChatSession([], `${DEFAULT_CHAT_SESSION_TITLE} ${nextSessions.length + 1}`)
      nextSessions = [...nextSessions, newSession]
      persistChatState(nextSessions, newSession.id)
      return {
        chatSessions: nextSessions,
        activeChatSessionId: newSession.id,
        chatMessages: [],
        isChatLoading: false,
      }
    })
  },

  setActiveChatSession: (sessionId) => {
    if (get().isChatLoading) return
    set((s) => {
      const target = s.chatSessions.find((session) => session.id === sessionId)
      if (!target) return {}
      persistChatState(s.chatSessions, target.id)
      return { activeChatSessionId: target.id, chatMessages: target.messages }
    })
  },

  closeChatSession: (sessionId) => {
    set((s) => {
      if (s.isChatLoading && s.activeChatSessionId === sessionId) return {}
      const idx = s.chatSessions.findIndex((session) => session.id === sessionId)
      if (idx < 0) return {}

      if (s.chatSessions.length <= 1) {
        const resetSession: ChatSession = { ...s.chatSessions[0], messages: [], updated_at: Date.now(), title: DEFAULT_CHAT_SESSION_TITLE }
        persistChatState([resetSession], resetSession.id)
        return {
          chatSessions: [resetSession],
          activeChatSessionId: resetSession.id,
          chatMessages: [],
          isChatLoading: false,
        }
      }

      const remaining = s.chatSessions.filter((session) => session.id !== sessionId)
      const nextActiveId = s.activeChatSessionId === sessionId
        ? remaining[Math.max(0, idx - 1)]?.id ?? remaining[0].id
        : (s.activeChatSessionId ?? remaining[0].id)
      const nextMessages = remaining.find((session) => session.id === nextActiveId)?.messages ?? []
      persistChatState(remaining, nextActiveId)
      return {
        chatSessions: remaining,
        activeChatSessionId: nextActiveId,
        chatMessages: nextMessages,
      }
    })
  },

  refreshChatMessagesForActiveUser: () => {
    const next = readStoredChatState()
    set({
      chatSessions: next.sessions,
      activeChatSessionId: next.activeSessionId,
      chatMessages: deriveActiveChatMessages(next.sessions, next.activeSessionId),
      isChatLoading: false,
    })
  },
  }
}
