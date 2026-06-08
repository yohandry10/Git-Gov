---
title: Resultados de Riesgo
description: Convierte la telemetría de gobernanza en resultados medibles de riesgo técnico para liderazgo y auditoría.
order: 9
category: Operate
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

Governance calcula un score ponderado con las señales disponibles:

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

## Perfiles por Tier

GitGov soporta perfiles por tier (`Critical`, `Standard`, `Internal`) en el modelo de reporting de gobernanza.

Cada perfil ajusta:
- pesos del score (readiness y riesgo compuesto),
- bandas de color para readiness,
- y objetivos SLA usados en alertas visuales de KPIs.

### Objetivos SLA base por tier

| Tier | Min readiness | Max push bloqueado | Max gap trazabilidad | Max fallos pipeline | Max fallos sonar | Max violaciones abiertas |
| --- | --- | --- | --- | --- | --- | --- |
| Critical | `>= 85` | `<= 5%` | `<= 15%` | `<= 10%` | `<= 12%` | `<= 30%` |
| Standard | `>= 75` | `<= 10%` | `<= 25%` | `<= 20%` | `<= 20%` | `<= 40%` |
| Internal | `>= 65` | `<= 15%` | `<= 35%` | `<= 30%` | `<= 30%` | `<= 50%` |

Usa `Standard` como baseline y mueve cada repositorio a `Critical` o `Internal` cuando tenga 2-4 semanas de telemetría estable.

---

## Patrón de Implementación Práctico

1. Definir baseline de cada KPI por tier de repositorio (backend crítico, tooling interno, etc.).
2. Medir tendencia semanal, no solo snapshots.
3. Asignar un owner por KPI (engineering manager o platform lead).
4. Mover un KPI a la vez con cambios de policy y quality gates.
5. Exportar evidencia mensual para gobernanza y auditoría.

---

## Métricas Operativas Ya Visibles

El reporting de Governance ya muestra dos métricas operativas informativas a partir de evidencia Jenkins correlacionada:

- **Time-to-Evidence** — desde el timestamp del commit hasta la ingesta del pipeline correlacionado.
- **MTTR pipeline** — desde un evento de pipeline no verde recuperable hasta el siguiente run exitoso del mismo job.

Estas métricas son muestrales y todavía no son garantías de producto respaldadas por SLO. No participan en el score compuesto de riesgo/readiness hasta calibrar umbrales SLO por tier.

## Iteración Pendiente

El trabajo restante es convertir esas métricas muestrales en SLOs calibrados y hacer que las tendencias de release readiness sean más fáciles de consumir por tier de repositorio. Ese trabajo debe alimentar el Enterprise Action Center, no crear otra cadena de reportes aislada.
