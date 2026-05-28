## storage-doctor: {{ env_label }}

Scan run at {{ generated_at }} UTC.

{% if has_rows -%}
**Unparsable rows ({{ total }})**

{% for r in rows -%}
- `{{ r.cf }}` ({{ r.source }}) at key `{{ r.key_hex }}`: {{ r.error }}
{% endfor %}
{%- else -%}
No unparsable rows found.
{% endif %}
---
