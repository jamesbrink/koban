#!/usr/bin/env bash
set -euo pipefail

if [ "${KOBAN_LIVE_WRITE_SMOKE:-}" != "1" ]; then
  echo "Set KOBAN_LIVE_WRITE_SMOKE=1 to run this mutating demo smoke test." >&2
  exit 2
fi

readonly INVOICE_NINJA_BASE_URL="https://demo.invoiceninja.com"
readonly INVOICE_NINJA_API_TOKEN="TOKEN"
export INVOICE_NINJA_BASE_URL INVOICE_NINJA_API_TOKEN

echo "Using Invoice Ninja public demo API: $INVOICE_NINJA_BASE_URL"

run_json() {
  cargo run --quiet -- --output json "$@"
}

run_table() {
  cargo run --quiet -- "$@"
}

created_ids=()
cleanup() {
  for id in "${created_ids[@]:-}"; do
    run_json invoices delete "$id" --yes >/dev/null 2>&1 || true
  done
}
trap cleanup EXIT

echo "== non-api commands =="
run_table --version
run_table --help >/tmp/koban-help-smoke.txt
run_table invoices --help >/tmp/koban-invoices-help-smoke.txt
for shell in bash zsh fish nushell elvish powershell; do
  run_table completions "$shell" >/tmp/koban-completion-"$shell"
  bytes=$(wc -c </tmp/koban-completion-"$shell")
  echo "completion_$shell bytes=$bytes"
done

echo "== statics =="
run_json statics >/tmp/koban-statics-smoke.json
printf "statics_keys=%s\n" "$(jq "keys | length" /tmp/koban-statics-smoke.json)"

resources=(clients invoices payments quotes credits vendors expenses projects tasks)
for resource in "${resources[@]}"; do
  echo "== $resource read commands =="
  run_json "$resource" list --per-page 1 >/tmp/koban-"$resource"-list.json
  row_count=$(jq ".data | length" /tmp/koban-"$resource"-list.json)
  echo "$resource list_rows=$row_count"
  id=$(jq -r ".data[0].id // empty" /tmp/koban-"$resource"-list.json)
  if [ -n "$id" ]; then
    run_json "$resource" show "$id" >/tmp/koban-"$resource"-show.json
    echo "$resource show_id=$(jq -r ".data.id // .id // empty" /tmp/koban-"$resource"-show.json)"
    run_json "$resource" edit-template "$id" >/tmp/koban-"$resource"-edit-template.json
    echo "$resource edit_template_type=$(jq -r "if .data then (.data|type) else type end" /tmp/koban-"$resource"-edit-template.json)"
  else
    echo "$resource show/edit-template skipped=no_rows"
  fi
  run_json "$resource" template >/tmp/koban-"$resource"-template.json
  echo "$resource template_type=$(jq -r "if .data then (.data|type) else type end" /tmp/koban-"$resource"-template.json)"
done

echo "== invoice write commands =="
client_id=$(jq -r ".data[0].id // empty" /tmp/koban-clients-list.json)
if [ -z "$client_id" ]; then
  echo "No demo client available." >&2
  exit 1
fi

run_json invoices create \
  --client-id "$client_id" \
  --line-item product_key=KobanSmoke,quantity=1,cost=1 \
  --dry-run >/tmp/koban-create-dry-run.json
echo "create_dry_run=$(jq -r .dry_run /tmp/koban-create-dry-run.json)"

run_json invoices delete dry_run_invoice --dry-run >/tmp/koban-delete-dry-run.json
echo "delete_dry_run=$(jq -r .dry_run /tmp/koban-delete-dry-run.json)"

run_json invoices bulk --action archive --id dry_one --id dry_two --dry-run >/tmp/koban-bulk-dry-run.json
echo "bulk_dry_run=$(jq -r .dry_run /tmp/koban-bulk-dry-run.json)"

run_json invoices action dry_invoice --action mark_paid --dry-run >/tmp/koban-action-dry-run.json
echo "action_dry_run=$(jq -r .dry_run /tmp/koban-action-dry-run.json)"

upload_dry_file=$(mktemp /tmp/koban-upload-dry.XXXXXX.txt)
printf "dry upload\n" >"$upload_dry_file"
run_json invoices upload dry_invoice --file "$upload_dry_file" --dry-run >/tmp/koban-upload-dry-run.json
echo "upload_dry_run=$(jq -r .dry_run /tmp/koban-upload-dry-run.json) method=$(jq -r .method /tmp/koban-upload-dry-run.json)"

echo "== expanded api dry-run commands =="
run_json products create --name KobanSmokeProduct --price 1 --dry-run >/tmp/koban-product-create-dry-run.json
echo "product_create_dry_run=$(jq -r .dry_run /tmp/koban-product-create-dry-run.json)"
run_json products update product_dry --field notes=Updated --dry-run >/tmp/koban-product-update-dry-run.json
echo "product_update_dry_run=$(jq -r .dry_run /tmp/koban-product-update-dry-run.json)"
run_json products delete product_dry --dry-run >/tmp/koban-product-delete-dry-run.json
echo "product_delete_dry_run=$(jq -r .dry_run /tmp/koban-product-delete-dry-run.json)"
run_json purchase-orders action po_dry --action email --dry-run >/tmp/koban-purchase-order-action-dry-run.json
echo "purchase_order_action_dry_run=$(jq -r .dry_run /tmp/koban-purchase-order-action-dry-run.json)"
run_json search run --field query=KobanSmoke --dry-run >/tmp/koban-search-dry-run.json
echo "search_dry_run=$(jq -r .dry_run /tmp/koban-search-dry-run.json)"

invoice_id=$(
  run_json invoices create \
    --client-id "$client_id" \
    --line-item product_key=KobanSmoke,quantity=1,cost=1 \
    --private-notes "Koban full smoke" |
    jq -r ".data.id // .id // empty"
)
if [ -z "$invoice_id" ]; then
  echo "Invoice creation did not return an id." >&2
  exit 1
fi
created_ids+=("$invoice_id")
echo "created=$invoice_id"

run_json invoices show "$invoice_id" --include client >/tmp/koban-created-show.json
echo "created_show=$(jq -r ".data.id // .id // empty" /tmp/koban-created-show.json)"
run_json invoices edit-template "$invoice_id" >/tmp/koban-created-edit-template.json
echo "created_edit_template=$(jq -r ".data.id // .id // empty" /tmp/koban-created-edit-template.json)"

payload=$(mktemp /tmp/koban-update.XXXXXX.json)
printf %s "{\"private_notes\":\"Koban full smoke updated\"}" >"$payload"
run_json invoices update "$invoice_id" --data-file "$payload" >/tmp/koban-updated.json
echo "updated=$(jq -r ".data.id // .id // empty" /tmp/koban-updated.json)"

upload_file=$(mktemp /tmp/koban-upload.XXXXXX.txt)
printf "koban full smoke upload\n" >"$upload_file"
run_json invoices upload "$invoice_id" --file "$upload_file" --yes >/tmp/koban-uploaded.json
echo "uploaded_type=$(jq -r "if .data then (.data|type) else type end" /tmp/koban-uploaded.json)"

run_json invoices action "$invoice_id" --action mark_paid --yes >/tmp/koban-action-mark-paid.json
echo "action_id=$(jq -r ".data.id // .id // empty" /tmp/koban-action-mark-paid.json)"

invitation_key=$(jq -r ".data.invitations[0].key // .invitations[0].key // empty" /tmp/koban-created-show.json)
if [ -n "$invitation_key" ]; then
  invoice_pdf=/tmp/koban-invoice-smoke-$$.pdf
  delivery_pdf=/tmp/koban-delivery-smoke-$$.pdf
  rm -f "$invoice_pdf" "$delivery_pdf"
  run_table invoices download "$invitation_key" --output-file "$invoice_pdf" >/tmp/koban-download.out
  run_table invoices delivery-note "$invoice_id" --output-file "$delivery_pdf" >/tmp/koban-delivery.out
  echo "download_bytes=$(wc -c <"$invoice_pdf") delivery_bytes=$(wc -c <"$delivery_pdf")"
else
  echo "download skipped=no_invitation_key"
fi

bulk_one=$(
  run_json invoices create \
    --client-id "$client_id" \
    --line-item product_key=KobanSmokeBulk,quantity=1,cost=1 \
    --private-notes "Koban bulk smoke 1" |
    jq -r ".data.id // .id // empty"
)
bulk_two=$(
  run_json invoices create \
    --client-id "$client_id" \
    --line-item product_key=KobanSmokeBulk,quantity=1,cost=1 \
    --private-notes "Koban bulk smoke 2" |
    jq -r ".data.id // .id // empty"
)
if [ -z "$bulk_one" ] || [ -z "$bulk_two" ]; then
  echo "Bulk smoke invoice creation failed." >&2
  exit 1
fi
created_ids+=("$bulk_one" "$bulk_two")
run_json invoices bulk --action archive --id "$bulk_one" --id "$bulk_two" --yes >/tmp/koban-bulk.json
echo "bulk_type=$(jq -r "if .data then (.data|type) else type end" /tmp/koban-bulk.json)"

for id in "${created_ids[@]}"; do
  run_json invoices delete "$id" --yes >/dev/null
  echo "deleted=$id"
done
created_ids=()

echo "== all command smoke complete =="
