---
title: Resultados de Riesgo
description: Convierte la telemetría de gobernanza en resultados medibles de riesgo técnico para liderazgo y auditoría.
order: 9
---

GitGov ya captura señales de políticas, CI y trazabilidad. El siguiente paso es mostrarlas como **resultados de negocio** que liderazgo técnico y compliance puedan seguir en el tiempo.

Esta página define el modelo KPI actual del Control Plane y cómo interpretar cada señal.

---

## Por Qué Importa

En entornos enterprise no se compran solo logs. Se compra:

- menor riesgo de entrega,
- evidencia de auditoría más rápida,
- y comportamiento de gobernanza predecible.

Los resultados de riesgo traducen eventos técnicos a un lenguaje accionable para seguridad, compliance y dirección.

---

## KPIs Base (Actuales)

### 1. Ruta Confiable

Qué porcentaje de actividad de entrega sigue la ruta gobernada esperada.

Fórmula:

```text
trusted_path_rate = tracked_pushes / (tracked_pushes + blocked_pushes)
```

Interpretación:
- Más alto es mejor.
- Una caída suele indicar fricción de políticas, intentos inseguros o brechas de onboarding.

---

### 2. Tasa de Push Bloqueado

Con qué frecuencia la política bloquea intentos de push.

Fórmula:

```text
blocked_push_rate = blocked_pushes / (tracked_pushes + blocked_pushes)
```

Interpretación:
- Valores moderados pueden ser normales durante el rollout.
- Valores altos sostenidos indican desalineación de política o patrones inseguros repetidos.

---

### 3. Brecha de Trazabilidad

Porcentaje de commits sin correlación con ticket.

Fórmula:

```text
traceability_gap = 100 - ticket_coverage_percentage
```

Interpretación:
- Más bajo es mejor.
- Valores altos reducen defendibilidad de auditoría y accountability de release.

---

### 4. Tasa de Fallo de Pipeline (7d)

Inestabilidad de builds en la ventana reciente de 7 días.

Fórmula:

```text
pipeline_failure_rate = failed_pipelines_7d / total_pipelines_7d
```

Interpretación:
- Se correlaciona con riesgo operativo y retrabajo.

---

### 5. Tasa de Fallo Sonar (Muestra)

Fallos de quality gate sobre runs de pipeline correlacionados con Sonar.

Fórmula:

```text
sonar_failure_rate = sonar_failed_runs / sonar_total_runs
```

Interpretación:
- Cuantifica regresiones de calidad y deuda técnica en código nuevo.

---

### 6. Tasa de Violaciones Abiertas

Violaciones de gobernanza pendientes sobre el total de violaciones.

Fórmula:

```text
unresolved_violation_rate = unresolved_violations / total_violations
```

Interpretación:
- Mide presión de backlog en workflows de respuesta y cumplimiento.

---

## Score Compuesto de Riesgo (0–100)

El dashboard calcula un score ponderado con las señales disponibles:

- tasa de push bloqueado,
- brecha de trazabilidad,
- tasa de fallo de pipeline,
- tasa de fallo Sonar,
- tasa de violaciones abiertas.

Cuando faltan señales, GitGov muestra **cobertura de señales** (`n/5`) para contextualizar la confianza del score.

---

## Bandas Operativas (Recomendadas)

- **Riesgo Bajo**: `< 35`
- **Riesgo Medio**: `35–59`
- **Riesgo Alto**: `>= 60`

Son umbrales operativos iniciales. Ajusta por criticidad de repositorio cuando tengas telemetría estable.

---

## Patrón de Implementación Práctico

1. Definir baseline de cada KPI por tier de repositorio (backend crítico, tooling interno, etc.).
2. Medir tendencia semanal, no solo snapshots.
3. Asignar un owner por KPI (engineering manager o platform lead).
4. Mover un KPI a la vez con cambios de policy y quality gates.
5. Exportar evidencia mensual para gobernanza y auditoría.

---

## Siguiente Iteración

Métricas de roadmap en implementación:

- MTTR de resolución de no cumplimiento,
- time-to-evidence para auditorías,
- tendencia de release readiness por tier de repositorio.

Estas mejoras se incorporarán sin romper contratos de eventos existentes.
