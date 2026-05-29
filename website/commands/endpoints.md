# Endpoint runners

Beyond the resource families, koban exposes runners for Invoice Ninja's
report, chart, search, and utility endpoints:

```sh
koban search run --field query=acme --dry-run
koban reports run --endpoint reports/invoices --data-file report.json --dry-run
koban charts run --endpoint charts/totals --data-file chart.json --dry-run
koban utility run
```

`search run` defaults to `POST /api/v1/search`, and `utility run` defaults to
safe `GET /api/v1/ping`. Reports and charts are grouped endpoints in the
official API, so pass the concrete endpoint path with `--endpoint`, for example
`reports/invoices` or `charts/totals`.

## Payload rules

Endpoint runner payload flags are accepted only for `POST` and `PUT` requests.
`GET` and `DELETE` reject payloads, so a dry-run can never show a body that the
live request would silently ignore.

Search, report, and chart endpoint runners default to `POST` when the endpoint
is not overridden. Utility and custom endpoint overrides are read-only.

## Custom endpoint overrides are read-only

Custom `--endpoint` overrides only send `GET` requests. This keeps the escape
hatch safe: use the first-class resource commands for any mutation.

```sh
# A custom override is GET-only.
koban utility run --endpoint some/read-only/path --output json
```

See [Safety](/guide/safety) for the complete guardrail summary.
