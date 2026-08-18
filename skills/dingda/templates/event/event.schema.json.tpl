{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "$id": "dingda://{{FEATURE}}/event/{{NAME}}/v1",
  "type": "object",
  "required": ["event_id", "occurred_at"],
  "properties": {
    "event_id": { "type": "string", "format": "uuid" },
    "occurred_at": { "type": "string", "format": "date-time" }
  },
  "additionalProperties": false
}
