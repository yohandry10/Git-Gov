import { Archive, Play, RefreshCw, Save, Settings2 } from 'lucide-react'
import { useEffect, useState } from 'react'
import { Badge } from '@/components/shared/Badge'
import { Button } from '@/components/shared/Button'
import { formatTs } from '@/lib/timezone'
import { useControlPlaneStore, type CompliancePeriodReportProfileRecord } from '@/store/useControlPlaneStore'

interface CompliancePeriodReportProfilePanelProps {
  selectedFrameworkId?: string | null
  dateRangeStart: number
  dateRangeEnd: number
  displayTimezone: string
}

function profileStatusVariant(status: string): 'success' | 'warning' | 'danger' | 'neutral' {
  if (status === 'active') return 'success'
  if (status === 'archived') return 'neutral'
  return 'warning'
}

function defaultProfileName(periodType: string): string {
  return `${periodType[0]?.toUpperCase() ?? 'M'}${periodType.slice(1)} evidence profile`
}

export function CompliancePeriodReportProfilePanel({
  selectedFrameworkId,
  dateRangeStart,
  dateRangeEnd,
  displayTimezone,
}: CompliancePeriodReportProfilePanelProps) {
  const [name, setName] = useState(defaultProfileName('monthly'))
  const [periodType, setPeriodType] = useState('monthly')
  const [includePdf, setIncludePdf] = useState(true)
  const [includeManifest, setIncludeManifest] = useState(true)
  const [retentionDays, setRetentionDays] = useState(2555)
  const [editingProfileId, setEditingProfileId] = useState<string | null>(null)

  const profiles = useControlPlaneStore((state) => state.compliancePeriodReportProfiles)
  const isCreating = useControlPlaneStore((state) => state.isCompliancePeriodReportProfileCreating)
  const isLoading = useControlPlaneStore((state) => state.isCompliancePeriodReportProfilesLoading)
  const isUpdating = useControlPlaneStore((state) => state.isCompliancePeriodReportProfileUpdating)
  const isArchiving = useControlPlaneStore((state) => state.isCompliancePeriodReportProfileArchiving)
  const isRunning = useControlPlaneStore((state) => state.isCompliancePeriodReportProfileRunning)
  const userRole = useControlPlaneStore((state) => state.userRole)
  const createProfile = useControlPlaneStore((state) => state.createCompliancePeriodReportProfile)
  const loadProfiles = useControlPlaneStore((state) => state.loadCompliancePeriodReportProfiles)
  const updateProfile = useControlPlaneStore((state) => state.updateCompliancePeriodReportProfile)
  const archiveProfile = useControlPlaneStore((state) => state.archiveCompliancePeriodReportProfile)
  const runProfile = useControlPlaneStore((state) => state.runCompliancePeriodReportProfile)

  const isAdmin = userRole === 'Admin'
  const normalizedName = name.trim()
  const canSave = isAdmin && normalizedName.length > 0 && retentionDays >= 30 && retentionDays <= 3650

  useEffect(() => {
    void loadProfiles({
      framework_id: selectedFrameworkId || null,
      status: 'active',
      limit: 10,
    })
  }, [loadProfiles, selectedFrameworkId])

  const resetForm = () => {
    setEditingProfileId(null)
    setName(defaultProfileName(periodType))
    setIncludePdf(true)
    setIncludeManifest(true)
    setRetentionDays(2555)
  }

  const handleSelectProfile = (profile: CompliancePeriodReportProfileRecord) => {
    setEditingProfileId(profile.profile_id)
    setName(profile.name)
    setPeriodType(profile.period_type)
    setIncludePdf(profile.include_pdf)
    setIncludeManifest(profile.include_manifest)
    setRetentionDays(profile.retention_days)
  }

  const handleSave = async () => {
    const payload = {
      name: normalizedName,
      period_type: periodType,
      framework_id: selectedFrameworkId || null,
      framework_owner_type: selectedFrameworkId ? 'gitgov_managed' : null,
      include_pdf: includePdf,
      include_manifest: includeManifest,
      retention_days: retentionDays,
      filters: {
        manual_run_template: true,
        selected_framework_id: selectedFrameworkId || null,
      },
    }
    if (editingProfileId) {
      await updateProfile(editingProfileId, payload)
    } else {
      await createProfile(payload)
    }
  }

  const handleRun = async (profile: CompliancePeriodReportProfileRecord) => {
    await runProfile(
      profile.profile_id,
      profile.period_type === 'custom'
        ? {
          date_range_start: dateRangeStart,
          date_range_end: dateRangeEnd,
        }
        : {},
    )
  }

  const handleLoad = async () => {
    await loadProfiles({
      framework_id: selectedFrameworkId || null,
      status: 'active',
      limit: 10,
    })
  }

  return (
    <div className="mt-2 rounded border border-white/8 bg-surface-950 p-2">
      <div className="flex flex-wrap items-center justify-between gap-2">
        <div className="flex items-center gap-2 text-xs font-medium text-surface-200">
          <Settings2 size={13} className="text-brand-300" />
          Saved report profiles
          <Badge variant="neutral">{profiles?.count ?? 0}</Badge>
        </div>
        <Button
          size="sm"
          variant="outline"
          loading={isLoading}
          onClick={() => void handleLoad()}
          title="Load saved period report profiles"
        >
          <RefreshCw size={13} />
          Load
        </Button>
      </div>

      {isAdmin && (
        <div className="mt-2 grid gap-2 md:grid-cols-[minmax(180px,1fr)_128px_98px_98px_110px_auto]">
          <input
            className="h-8 min-w-0 rounded border border-white/10 bg-surface-900 px-2 text-xs text-surface-100 outline-none focus:border-brand-400"
            value={name}
            maxLength={120}
            onChange={(event) => setName(event.target.value)}
            aria-label="Profile name"
          />
          <select
            className="h-8 rounded border border-white/10 bg-surface-900 px-2 text-xs text-surface-100 outline-none focus:border-brand-400"
            value={periodType}
            onChange={(event) => {
              setPeriodType(event.target.value)
              if (!editingProfileId) setName(defaultProfileName(event.target.value))
            }}
            aria-label="Profile period type"
          >
            <option value="monthly">Monthly</option>
            <option value="quarterly">Quarterly</option>
            <option value="annual">Annual</option>
            <option value="custom">Custom</option>
          </select>
          <label className="flex h-8 items-center gap-2 rounded border border-white/10 bg-surface-900 px-2 text-[11px] text-surface-300">
            <input
              type="checkbox"
              checked={includePdf}
              onChange={(event) => setIncludePdf(event.target.checked)}
            />
            PDF
          </label>
          <label className="flex h-8 items-center gap-2 rounded border border-white/10 bg-surface-900 px-2 text-[11px] text-surface-300">
            <input
              type="checkbox"
              checked={includeManifest}
              onChange={(event) => setIncludeManifest(event.target.checked)}
            />
            Manifest
          </label>
          <input
            className="h-8 rounded border border-white/10 bg-surface-900 px-2 text-xs text-surface-100 outline-none focus:border-brand-400"
            type="number"
            min={30}
            max={3650}
            value={retentionDays}
            onChange={(event) => setRetentionDays(Number(event.target.value))}
            aria-label="Profile retention days"
          />
          <div className="flex gap-2">
            <Button
              size="sm"
              variant="outline"
              loading={isCreating || isUpdating}
              disabled={!canSave}
              onClick={() => void handleSave()}
              title={editingProfileId ? 'Update saved report profile' : 'Create saved report profile'}
            >
              <Save size={13} />
              {editingProfileId ? 'Save' : 'Create'}
            </Button>
            {editingProfileId && (
              <Button
                size="sm"
                variant="ghost"
                onClick={resetForm}
                title="Clear selected profile form"
              >
                New
              </Button>
            )}
          </div>
        </div>
      )}

      {profiles && profiles.items.length > 0 && (
        <div className="mt-2 space-y-2">
          {profiles.items.slice(0, 10).map((profile) => (
            <div key={profile.profile_id} className="grid gap-2 rounded border border-white/6 bg-white/[0.02] p-2 text-[11px] md:grid-cols-[minmax(180px,1fr)_auto]">
              <button
                type="button"
                className="min-w-0 text-left"
                onClick={() => handleSelectProfile(profile)}
                title="Load this profile into the form"
              >
                <div className="flex flex-wrap items-center gap-2">
                  <span className="truncate text-xs font-medium text-surface-200">{profile.name}</span>
                  <Badge variant={profileStatusVariant(profile.status)}>{profile.status}</Badge>
                  <Badge variant="info">{profile.period_type}</Badge>
                  {profile.include_pdf && <Badge variant="neutral">PDF</Badge>}
                  {profile.include_manifest && <Badge variant="neutral">Manifest</Badge>}
                </div>
                <div className="mt-1 flex flex-wrap gap-x-3 gap-y-1 text-surface-500">
                  <span>{profile.retention_days} retention days</span>
                  <span>{profile.run_count} runs</span>
                  <span>{profile.last_run_at ? formatTs(profile.last_run_at, displayTimezone) : 'never run'}</span>
                  <span>{profile.framework_id ?? 'all frameworks'}</span>
                </div>
              </button>
              <div className="flex flex-wrap items-center justify-end gap-2">
                {isAdmin && (
                  <Button
                    size="sm"
                    variant="outline"
                    loading={isRunning}
                    onClick={() => void handleRun(profile)}
                    title="Run this saved report profile now"
                  >
                    <Play size={13} />
                    Run now
                  </Button>
                )}
                {isAdmin && (
                  <Button
                    size="sm"
                    variant="ghost"
                    loading={isArchiving}
                    onClick={() => void archiveProfile(profile.profile_id)}
                    title="Archive this saved report profile"
                  >
                    <Archive size={13} />
                    Archive
                  </Button>
                )}
              </div>
            </div>
          ))}
        </div>
      )}
    </div>
  )
}
