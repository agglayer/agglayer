## {{ env_label }}

Migration run at {{ started_at }} UTC, total {{ overall_duration }}. Status: **{{ status_label }}**.

| Store | Status | Duration | Notes |
|---|---|---:|---|
{%- for row in rows %}
| {{ row.label }} | {{ row.status }} | {{ row.duration }} | {{ row.notes }} |
{%- endfor %}

{% if !is_success -%}
**Fatal errors**

{% for fatal in fatals -%}
- {{ fatal.label }}: {{ fatal.error }}
{% endfor %}
{% endif -%}
{% if !diagnostics_warnings.is_empty() -%}
**Diagnostics warnings**

{% for warning in diagnostics_warnings -%}
- epoch {{ warning.epoch }}: {{ warning.error }}
{% endfor %}
{% endif -%}
{% if has_unparsable -%}
**Unparsable rows ({{ unparsable_count }})**

{% for u in unparsable -%}
- `{{ u.cf }}` ({{ u.source }}) at key `{{ u.key_hex }}`: {{ u.error }}
{% endfor %}
{% endif -%}
---
