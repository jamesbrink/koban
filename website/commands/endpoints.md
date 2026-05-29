# Endpoint runners

Beyond the resource families, koban exposes runners for Invoice Ninja's
report, chart, search, and utility endpoints:

```sh
koban search run --field query=acme --dry-run
koban reports run --data-file report.json --dry-run
koban charts run
koban utility run
```

## Payload rules

Endpoint runner payload flags are accepted only for `POST` and `PUT` requests.
`GET` and `DELETE` reject payloads, so a dry-run can never show a body that the
live request would silently ignore.

Generic endpoint runner defaults may use non-`GET` methods with `--yes`.

## Custom endpoint overrides are read-only

Custom `--endpoint` overrides only send `GET` requests. This keeps the escape
hatch safe: use the first-class resource commands for any mutation.

```sh
# A custom override is GET-only.
koban utility run --endpoint some/read-only/path --output json
```

See [Safety](/guide/safety) for the complete guardrail summary.
