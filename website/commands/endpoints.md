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
is not overridden. Report overrides under `reports/...` and chart overrides
under `charts/...` may use payload methods because those are the concrete
official grouped endpoints. Utility and other custom endpoint overrides are
read-only.

## Custom endpoint overrides are read-only

Custom `--endpoint` overrides outside the report/chart grouped endpoint
families only send `GET` requests. This keeps the escape hatch safe: use the
first-class resource commands for any mutation.

```sh
# A custom override is GET-only.
koban utility run --endpoint some/read-only/path --output json
```

See [Safety](/guide/safety) for the complete guardrail summary.
