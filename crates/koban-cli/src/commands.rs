use serde_json::{Value, json};

use koban::{ApiClient, Config, KobanError, Resource, Result};

use crate::{
    auth,
    cli::{
        BulkArgs, Cli, Commands, ConfirmableIdArgs, DownloadArgs, HttpMethod,
        InspectResourceCommand, InvoiceActionArgs, InvoiceCommand, InvoiceWriteArgs, ListArgs,
        OutputFormat, ResourceActionArgs, ResourceCommand, ResourceWriteArgs, UpdateInvoiceArgs,
        UpdateResourceArgs, UploadArgs,
    },
    config_store,
    endpoint_runner::execute_endpoint,
    invoice::{
        invoice_payload, push_invoice_triggers, render_dry_run, require_confirmation,
        validate_invoice_triggers, validate_path_segment,
    },
    payload::{merge_resource_action_payload, resource_payload},
    render::{render_value, response_rows},
    resource_routes::{
        ResourceCapability, require_resource_capability, resource_action_route,
        resource_delete_path, resource_download_base_path, resource_update_path,
    },
    skill, update,
};

pub(crate) use crate::file_paths::{ensure_download_path, ensure_upload_file, write_download_file};

const FETCH_ALL_PAGE_CAP: u32 = 100;

pub async fn execute(cli: Cli) -> Result<String> {
    let output = cli.output;
    let command = cli.command;

    match command {
        Some(Commands::Update {
            check,
            force,
            tag,
            nightly,
        }) => update::run(check, force, tag, nightly),
        // Auth and skill commands run without a resolved token: login is how a
        // token is obtained, and the skill generator describes the CLI offline.
        Some(Commands::Auth(command)) => auth::execute(output, command).await,
        Some(Commands::Skill(command)) => skill::execute(output, command),
        command => {
            let (base_url, token) = config_store::resolve()?;
            let config = Config::from_values(base_url, token)?;
            execute_with_config(Cli { output, command }, config).await
        }
    }
}

pub async fn execute_with_config(cli: Cli, config: Config) -> Result<String> {
    let client = ApiClient::new(config);
    let output = cli.output;

    match cli.command {
        Some(Commands::Statics) => {
            let json = client.get_json("api/v1/statics", &[]).await?;
            render_value(output, None, &json)
        }
        Some(Commands::Clients(command)) => {
            execute_resource(&client, output, Resource::Clients, command).await
        }
        Some(Commands::Invoices(command)) => execute_invoice(&client, output, *command).await,
        Some(Commands::Payments(command)) => {
            execute_resource(&client, output, Resource::Payments, command).await
        }
        Some(Commands::Quotes(command)) => {
            execute_resource(&client, output, Resource::Quotes, command).await
        }
        Some(Commands::Credits(command)) => {
            execute_resource(&client, output, Resource::Credits, command).await
        }
        Some(Commands::Vendors(command)) => {
            execute_resource(&client, output, Resource::Vendors, command).await
        }
        Some(Commands::Expenses(command)) => {
            execute_resource(&client, output, Resource::Expenses, command).await
        }
        Some(Commands::Projects(command)) => {
            execute_resource(&client, output, Resource::Projects, command).await
        }
        Some(Commands::Tasks(command)) => {
            execute_resource(&client, output, Resource::Tasks, command).await
        }
        Some(Commands::Locations(command)) => {
            execute_resource(&client, output, Resource::Locations, command).await
        }
        Some(Commands::Products(command)) => {
            execute_resource(&client, output, Resource::Products, command).await
        }
        Some(Commands::RecurringInvoices(command)) => {
            execute_resource(&client, output, Resource::RecurringInvoices, command).await
        }
        Some(Commands::PurchaseOrders(command)) => {
            execute_resource(&client, output, Resource::PurchaseOrders, command).await
        }
        Some(Commands::RecurringExpenses(command)) => {
            execute_resource(&client, output, Resource::RecurringExpenses, command).await
        }
        Some(Commands::RecurringQuotes(command)) => {
            execute_resource(&client, output, Resource::RecurringQuotes, command).await
        }
        Some(Commands::BankTransactions(command)) => {
            execute_resource(&client, output, Resource::BankTransactions, command).await
        }
        Some(Commands::BankIntegrations(command)) => {
            execute_resource(&client, output, Resource::BankIntegrations, command).await
        }
        Some(Commands::BankTransactionRules(command)) => {
            execute_resource(&client, output, Resource::BankTransactionRules, command).await
        }
        Some(Commands::GroupSettings(command)) => {
            execute_resource(&client, output, Resource::GroupSettings, command).await
        }
        Some(Commands::ExpenseCategories(command)) => {
            execute_resource(&client, output, Resource::ExpenseCategories, command).await
        }
        Some(Commands::TaxRates(command)) => {
            execute_resource(&client, output, Resource::TaxRates, command).await
        }
        Some(Commands::PaymentTerms(command)) => {
            execute_resource(&client, output, Resource::PaymentTerms, command).await
        }
        Some(Commands::TaskSchedulers(command)) => {
            execute_resource(&client, output, Resource::TaskSchedulers, command).await
        }
        Some(Commands::TaskStatuses(command)) => {
            execute_resource(&client, output, Resource::TaskStatuses, command).await
        }
        Some(Commands::Activities(command)) => {
            execute_inspect_resource(&client, output, Resource::Activities, command).await
        }
        Some(Commands::SystemLogs(command)) => {
            execute_inspect_resource(&client, output, Resource::SystemLogs, command).await
        }
        Some(Commands::Documents(command)) => {
            execute_resource(&client, output, Resource::Documents, command).await
        }
        Some(Commands::Designs(command)) => {
            execute_resource(&client, output, Resource::Designs, command).await
        }
        Some(Commands::Templates(command)) => {
            execute_resource(&client, output, Resource::Templates, command).await
        }
        Some(Commands::Users(command)) => {
            execute_resource(&client, output, Resource::Users, command).await
        }
        Some(Commands::Companies(command)) => {
            execute_resource(&client, output, Resource::Companies, command).await
        }
        Some(Commands::CompanyGateways(command)) => {
            execute_resource(&client, output, Resource::CompanyGateways, command).await
        }
        Some(Commands::CompanyLedger(command)) => {
            execute_inspect_resource(&client, output, Resource::CompanyLedger, command).await
        }
        Some(Commands::CompanyUsers(command)) => {
            execute_resource(&client, output, Resource::CompanyUsers, command).await
        }
        Some(Commands::Tokens(command)) => {
            execute_resource(&client, output, Resource::Tokens, command).await
        }
        Some(Commands::Webhooks(command)) => {
            execute_resource(&client, output, Resource::Webhooks, command).await
        }
        Some(Commands::Subscriptions(command)) => {
            execute_resource(&client, output, Resource::Subscriptions, command).await
        }
        Some(Commands::ClientGatewayTokens(command)) => {
            execute_resource(&client, output, Resource::ClientGatewayTokens, command).await
        }
        Some(Commands::Reports(command)) => {
            execute_endpoint(&client, output, "reports", command).await
        }
        Some(Commands::Charts(command)) => {
            execute_endpoint(&client, output, "charts", command).await
        }
        Some(Commands::Search(command)) => {
            execute_endpoint(&client, output, "search", command).await
        }
        Some(Commands::Utility(command)) => {
            execute_endpoint(&client, output, "ping", command).await
        }
        Some(Commands::Update {
            check,
            force,
            tag,
            nightly,
        }) => update::run(check, force, tag, nightly),
        // Auth, Skill, and Completions are dispatched before a config is
        // resolved, so they never reach this point.
        Some(Commands::Auth(_)) | Some(Commands::Skill(_)) => Ok(String::new()),
        Some(Commands::Completions { .. }) | None => Ok(String::new()),
    }
}

async fn execute_resource(
    client: &ApiClient,
    output: OutputFormat,
    resource: Resource,
    command: ResourceCommand,
) -> Result<String> {
    match command {
        ResourceCommand::List(args) => execute_list(client, output, resource, args).await,
        ResourceCommand::Show(args) => {
            require_resource_capability(resource, ResourceCapability::Show)?;
            validate_path_id(&format!("{} id", resource.label()), &args.id)?;
            let mut query = Vec::new();
            push_include(&mut query, args.include);

            let json = client
                .get_json(&format!("api/v1/{}/{}", resource.path(), args.id), &query)
                .await?;
            render_value(output, Some(resource), &json)
        }
        ResourceCommand::Template(args) => {
            require_resource_capability(resource, ResourceCapability::Template)?;
            let mut query = Vec::new();
            push_include(&mut query, args.include);

            let json = client
                .get_json(&format!("api/v1/{}/create", resource.path()), &query)
                .await?;
            render_value(output, Some(resource), &json)
        }
        ResourceCommand::EditTemplate(args) => {
            require_resource_capability(resource, ResourceCapability::EditTemplate)?;
            validate_path_id(&format!("{} id", resource.label()), &args.id)?;
            let mut query = Vec::new();
            push_include(&mut query, args.include);

            let json = client
                .get_json(
                    &format!("api/v1/{}/{}/edit", resource.path(), args.id),
                    &query,
                )
                .await?;
            render_value(output, Some(resource), &json)
        }
        ResourceCommand::Create(args) => {
            execute_resource_create(client, output, resource, args).await
        }
        ResourceCommand::Update(args) => {
            execute_resource_update(client, output, resource, args).await
        }
        ResourceCommand::Delete(args) => {
            execute_resource_delete(client, output, resource, args).await
        }
        ResourceCommand::Bulk(args) => execute_resource_bulk(client, output, resource, args).await,
        ResourceCommand::Upload(args) => {
            execute_resource_upload(client, output, resource, args).await
        }
        ResourceCommand::Action(args) => {
            execute_resource_action(client, output, resource, args).await
        }
        ResourceCommand::Download(args) => execute_resource_download(client, resource, args).await,
    }
}

async fn execute_inspect_resource(
    client: &ApiClient,
    output: OutputFormat,
    resource: Resource,
    command: InspectResourceCommand,
) -> Result<String> {
    let command = match command {
        InspectResourceCommand::List(args) => ResourceCommand::List(args),
        InspectResourceCommand::Show(args) => ResourceCommand::Show(args),
    };
    execute_resource(client, output, resource, command).await
}

async fn execute_resource_create(
    client: &ApiClient,
    output: OutputFormat,
    resource: Resource,
    args: ResourceWriteArgs,
) -> Result<String> {
    require_resource_capability(resource, ResourceCapability::Create)?;
    let body = resource_payload(args.payload, true)?;
    let mut query = Vec::new();
    push_include(&mut query, args.include);
    let path = format!("api/v1/{}", resource.path());

    require_confirmation(&format!("{} create", resource.label()), &args.safety)?;

    if args.safety.dry_run {
        return render_dry_run("POST", &path, &query, Some(&body), None);
    }

    let json = client.post_json(&path, &query, &body).await?;
    render_value(output, Some(resource), &json)
}

async fn execute_resource_update(
    client: &ApiClient,
    output: OutputFormat,
    resource: Resource,
    args: UpdateResourceArgs,
) -> Result<String> {
    require_resource_capability(resource, ResourceCapability::Update)?;
    let body = resource_payload(args.payload, true)?;
    let mut query = Vec::new();
    push_include(&mut query, args.include);
    validate_path_id(&format!("{} id", resource.label()), &args.id)?;
    let path = resource_update_path(resource, &args.id);

    require_confirmation(&format!("{} update", resource.label()), &args.safety)?;

    if args.safety.dry_run {
        return render_dry_run("PUT", &path, &query, Some(&body), None);
    }

    let json = client.put_json(&path, &query, &body).await?;
    render_value(output, Some(resource), &json)
}

async fn execute_resource_delete(
    client: &ApiClient,
    output: OutputFormat,
    resource: Resource,
    args: ConfirmableIdArgs,
) -> Result<String> {
    require_resource_capability(resource, ResourceCapability::Delete)?;
    require_confirmation(&format!("{} delete", resource.label()), &args.safety)?;
    validate_path_id(&format!("{} id", resource.label()), &args.id)?;
    let mut query = Vec::new();
    push_include(&mut query, args.include);
    let path = resource_delete_path(resource, &args.id);

    if args.safety.dry_run {
        return render_dry_run("DELETE", &path, &query, None, None);
    }

    let json = client.delete_json(&path, &query).await?;
    render_value(output, Some(resource), &json)
}

async fn execute_resource_bulk(
    client: &ApiClient,
    output: OutputFormat,
    resource: Resource,
    args: BulkArgs,
) -> Result<String> {
    require_resource_capability(resource, ResourceCapability::Bulk)?;
    require_confirmation(&format!("{} bulk action", resource.label()), &args.safety)?;
    validate_path_ids(&format!("{} id", resource.label()), &args.ids)?;
    let mut query = Vec::new();
    push_include(&mut query, args.include);
    let body = bulk_action_body(args.action, args.ids, args.email_type);
    let path = format!("api/v1/{}/bulk", resource.path());

    if args.safety.dry_run {
        return render_dry_run("POST", &path, &query, Some(&body), None);
    }

    let json = client.post_json(&path, &query, &body).await?;
    render_value(output, Some(resource), &json)
}

async fn execute_resource_upload(
    client: &ApiClient,
    output: OutputFormat,
    resource: Resource,
    args: UploadArgs,
) -> Result<String> {
    require_resource_capability(resource, ResourceCapability::Upload)?;
    require_confirmation(
        &format!("{} document upload", resource.label()),
        &args.safety,
    )?;
    validate_path_id(&format!("{} id", resource.label()), &args.id)?;
    for file in &args.files {
        ensure_upload_file(file)?;
    }

    let mut query = Vec::new();
    push_include(&mut query, args.include);
    let path = format!("api/v1/{}/{}/upload", resource.path(), args.id);

    if args.safety.dry_run {
        return render_dry_run(
            resource.upload_method(),
            &path,
            &query,
            None,
            Some(&args.files),
        );
    }

    let json = if resource.upload_method() == "PUT" {
        client.put_multipart(&path, &query, &args.files).await?
    } else {
        client.post_multipart(&path, &query, &args.files).await?
    };
    render_value(output, Some(resource), &json)
}

async fn execute_resource_action(
    client: &ApiClient,
    output: OutputFormat,
    resource: Resource,
    args: ResourceActionArgs,
) -> Result<String> {
    require_confirmation(&format!("{} action", resource.label()), &args.safety)?;
    validate_path_id(&format!("{} id", resource.label()), &args.id)?;
    validate_path_segment("resource action", &args.action)?;
    let mut query = Vec::new();
    push_include(&mut query, args.include);
    let route = resource_action_route(resource, &args.id, &args.action);
    if route.is_bulk {
        require_resource_capability(resource, ResourceCapability::Bulk)?;
    }
    let body = resource_payload(
        args.payload,
        matches!(route.method, HttpMethod::Post | HttpMethod::Put),
    )?;
    if route.method == HttpMethod::Get && body.as_object().is_some_and(|body| !body.is_empty()) {
        return Err(KobanError::InvalidPayload {
            message: "GET resource actions do not send request bodies".to_string(),
        });
    }
    let bulk_body = if route.is_bulk {
        let mut body_with_action = bulk_action_body(args.action, vec![args.id], None);
        merge_resource_action_payload(&mut body_with_action, body);
        body_with_action
    } else {
        body
    };

    if args.safety.dry_run {
        return render_dry_run(
            route.method.label(),
            &route.path,
            &query,
            route.body.then_some(&bulk_body),
            None,
        );
    }

    let json = match route.method {
        HttpMethod::Get => client.get_json(&route.path, &query).await?,
        HttpMethod::Post => client.post_json(&route.path, &query, &bulk_body).await?,
        HttpMethod::Put => client.put_json(&route.path, &query, &bulk_body).await?,
        HttpMethod::Delete => client.delete_json(&route.path, &query).await?,
    };
    render_value(output, Some(resource), &json)
}

async fn execute_resource_download(
    client: &ApiClient,
    resource: Resource,
    args: DownloadArgs,
) -> Result<String> {
    let Some(base_path) = resource_download_base_path(resource) else {
        return Err(KobanError::InvalidPayload {
            message: format!("{} does not support PDF downloads", resource.label()),
        });
    };
    execute_download(client, base_path, "download", args).await
}

async fn execute_invoice(
    client: &ApiClient,
    output: OutputFormat,
    command: InvoiceCommand,
) -> Result<String> {
    match command {
        InvoiceCommand::List(args) => execute_list(client, output, Resource::Invoices, args).await,
        InvoiceCommand::Show(args) => {
            execute_resource(
                client,
                output,
                Resource::Invoices,
                ResourceCommand::Show(args),
            )
            .await
        }
        InvoiceCommand::Template(args) => {
            execute_resource(
                client,
                output,
                Resource::Invoices,
                ResourceCommand::Template(args),
            )
            .await
        }
        InvoiceCommand::EditTemplate(args) => {
            execute_resource(
                client,
                output,
                Resource::Invoices,
                ResourceCommand::EditTemplate(args),
            )
            .await
        }
        InvoiceCommand::Create(args) => execute_invoice_create(client, output, args).await,
        InvoiceCommand::Update(args) => execute_invoice_update(client, output, args).await,
        InvoiceCommand::Delete(args) => execute_invoice_delete(client, output, args).await,
        InvoiceCommand::Bulk(args) => execute_invoice_bulk(client, output, args).await,
        InvoiceCommand::Upload(args) => execute_invoice_upload(client, output, args).await,
        InvoiceCommand::Action(args) => execute_invoice_action(client, output, args).await,
        InvoiceCommand::Download(args) => {
            execute_download(client, "api/v1/invoice", "download", args).await
        }
        InvoiceCommand::DeliveryNote(args) => {
            execute_download(client, "api/v1/invoices", "delivery_note", args).await
        }
    }
}

async fn execute_list(
    client: &ApiClient,
    output: OutputFormat,
    resource: Resource,
    args: ListArgs,
) -> Result<String> {
    require_resource_capability(resource, ResourceCapability::List)?;
    let mut base_query = Vec::new();
    push_include(&mut base_query, args.include);
    push_sort(&mut base_query, args.sort);
    push_filters(&mut base_query, args.filters)?;

    if !args.all {
        let mut query = base_query;
        query.push(("page".to_string(), args.page.to_string()));
        query.push(("per_page".to_string(), args.per_page.to_string()));

        let json = client
            .get_json(&format!("api/v1/{}", resource.path()), &query)
            .await?;
        let json = apply_limit_to_response(json, args.limit);
        return render_value(output, Some(resource), &json);
    }

    let json = fetch_all_pages(
        client,
        resource,
        &base_query,
        args.page,
        args.per_page,
        args.limit,
    )
    .await?;
    render_value(output, Some(resource), &json)
}

async fn execute_invoice_create(
    client: &ApiClient,
    output: OutputFormat,
    args: InvoiceWriteArgs,
) -> Result<String> {
    validate_invoice_triggers(&args.triggers)?;
    let body = invoice_payload(args.payload, true, false)?;
    let mut query = Vec::new();
    push_include(&mut query, args.include);
    push_invoice_triggers(&mut query, &args.triggers);

    if args.triggers.requires_confirmation() {
        require_confirmation("invoice create with state-changing trigger", &args.safety)?;
    }

    if args.safety.dry_run {
        return render_dry_run("POST", "api/v1/invoices", &query, Some(&body), None);
    }

    let json = client.post_json("api/v1/invoices", &query, &body).await?;
    render_value(output, Some(Resource::Invoices), &json)
}

async fn execute_invoice_update(
    client: &ApiClient,
    output: OutputFormat,
    args: UpdateInvoiceArgs,
) -> Result<String> {
    validate_invoice_triggers(&args.triggers)?;
    let body = invoice_payload(args.payload, false, args.triggers.has_any())?;
    let mut query = Vec::new();
    push_include(&mut query, args.include);
    push_invoice_triggers(&mut query, &args.triggers);

    if args.triggers.requires_confirmation() {
        require_confirmation("invoice update with state-changing trigger", &args.safety)?;
    }

    validate_path_id("invoice id", &args.id)?;
    let path = format!("api/v1/invoices/{}", args.id);
    if args.safety.dry_run {
        return render_dry_run("PUT", &path, &query, Some(&body), None);
    }

    let json = client.put_json(&path, &query, &body).await?;
    render_value(output, Some(Resource::Invoices), &json)
}

async fn execute_invoice_delete(
    client: &ApiClient,
    output: OutputFormat,
    args: ConfirmableIdArgs,
) -> Result<String> {
    require_confirmation("invoice delete", &args.safety)?;
    validate_path_id("invoice id", &args.id)?;
    let mut query = Vec::new();
    push_include(&mut query, args.include);

    let path = format!("api/v1/invoices/{}", args.id);
    if args.safety.dry_run {
        return render_dry_run("DELETE", &path, &query, None, None);
    }

    let json = client.delete_json(&path, &query).await?;
    render_value(output, Some(Resource::Invoices), &json)
}

async fn execute_invoice_bulk(
    client: &ApiClient,
    output: OutputFormat,
    args: BulkArgs,
) -> Result<String> {
    require_confirmation("invoice bulk action", &args.safety)?;
    validate_path_ids("invoice id", &args.ids)?;
    let mut query = Vec::new();
    push_include(&mut query, args.include);
    let body = bulk_action_body(args.action, args.ids, args.email_type);

    if args.safety.dry_run {
        return render_dry_run("POST", "api/v1/invoices/bulk", &query, Some(&body), None);
    }

    let json = client
        .post_json("api/v1/invoices/bulk", &query, &body)
        .await?;
    render_value(output, Some(Resource::Invoices), &json)
}

fn bulk_action_body(action: String, ids: Vec<String>, email_type: Option<String>) -> Value {
    let mut body = serde_json::Map::new();
    body.insert("action".to_string(), Value::String(action));
    body.insert(
        "ids".to_string(),
        Value::Array(ids.into_iter().map(Value::String).collect()),
    );
    if let Some(email_type) = email_type {
        body.insert("email_type".to_string(), Value::String(email_type));
    }
    Value::Object(body)
}

async fn execute_invoice_upload(
    client: &ApiClient,
    output: OutputFormat,
    args: UploadArgs,
) -> Result<String> {
    require_confirmation("invoice document upload", &args.safety)?;
    validate_path_id("invoice id", &args.id)?;
    for file in &args.files {
        ensure_upload_file(file)?;
    }

    let mut query = Vec::new();
    push_include(&mut query, args.include);
    let path = format!("api/v1/invoices/{}/upload", args.id);

    if args.safety.dry_run {
        return render_dry_run("PUT", &path, &query, None, Some(&args.files));
    }

    let json = client.put_multipart(&path, &query, &args.files).await?;
    render_value(output, Some(Resource::Invoices), &json)
}

async fn execute_invoice_action(
    client: &ApiClient,
    output: OutputFormat,
    args: InvoiceActionArgs,
) -> Result<String> {
    require_confirmation("invoice action", &args.safety)?;
    validate_path_id("invoice id", &args.id)?;
    validate_path_segment("invoice action", &args.action)?;
    let mut query = Vec::new();
    push_include(&mut query, args.include);
    let path = format!("api/v1/invoices/{}/{}", args.id, args.action);

    if args.safety.dry_run {
        return render_dry_run("GET", &path, &query, None, None);
    }

    let json = client.get_json(&path, &query).await?;
    render_value(output, Some(Resource::Invoices), &json)
}

async fn execute_download(
    client: &ApiClient,
    base_path: &str,
    action: &str,
    args: DownloadArgs,
) -> Result<String> {
    let mut query = Vec::new();
    push_include(&mut query, args.include);
    validate_path_id("download id", &args.id)?;
    ensure_download_path(&args.output_file, args.force)?;
    write_download_file(
        &args.output_file,
        client
            .get_bytes(&format!("{base_path}/{}/{action}", args.id), &query)
            .await?,
        args.force,
    )?;
    Ok(format!("Wrote {}", args.output_file.display()))
}

async fn fetch_all_pages(
    client: &ApiClient,
    resource: Resource,
    base_query: &[(String, String)],
    start_page: u32,
    per_page: u32,
    limit: Option<u32>,
) -> Result<Value> {
    let mut page = start_page;
    let mut pages_fetched = 0_u32;
    let mut page_cap_reached = false;
    let mut rows = Vec::new();

    loop {
        let mut query = base_query.to_vec();
        query.push(("page".to_string(), page.to_string()));
        query.push(("per_page".to_string(), per_page.to_string()));

        let json = client
            .get_json(&format!("api/v1/{}", resource.path()), &query)
            .await?;
        let page_rows = response_rows(&json)
            .into_iter()
            .cloned()
            .collect::<Vec<_>>();
        let page_len = page_rows.len();
        pages_fetched += 1;

        for row in page_rows {
            if limit.is_some_and(|limit| rows.len() >= limit as usize) {
                break;
            }
            rows.push(row);
        }

        if page_len < per_page as usize || limit.is_some_and(|limit| rows.len() >= limit as usize) {
            break;
        }
        if pages_fetched >= FETCH_ALL_PAGE_CAP {
            page_cap_reached = true;
            break;
        }
        page += 1;
    }

    Ok(json!({
        "data": rows,
        "meta": {
            "pages_fetched": pages_fetched,
            "page_cap": FETCH_ALL_PAGE_CAP,
            "page_cap_reached": page_cap_reached,
            "limit": limit,
        }
    }))
}

fn validate_path_id(label: &str, id: &str) -> Result<()> {
    validate_path_segment(label, id)
}

fn validate_path_ids(label: &str, ids: &[String]) -> Result<()> {
    for id in ids {
        validate_path_id(label, id)?;
    }
    Ok(())
}

pub(crate) fn push_include(query: &mut Vec<(String, String)>, include: Vec<String>) {
    let include = include
        .into_iter()
        .map(|part| part.trim().to_string())
        .filter(|part| !part.is_empty())
        .collect::<Vec<_>>();

    if !include.is_empty() {
        query.push(("include".to_string(), include.join(",")));
    }
}

pub(crate) fn push_sort(query: &mut Vec<(String, String)>, sort: Option<String>) {
    if let Some(sort) = sort
        .map(|sort| sort.trim().to_string())
        .filter(|sort| !sort.is_empty())
    {
        query.push(("sort".to_string(), sort));
    }
}

pub(crate) fn push_filters(query: &mut Vec<(String, String)>, filters: Vec<String>) -> Result<()> {
    for filter in filters {
        let Some((key, value)) = filter.split_once('=') else {
            return Err(KobanError::InvalidFilter { value: filter });
        };
        let key = key.trim();
        if key.is_empty() {
            return Err(KobanError::InvalidFilter { value: filter });
        }
        query.push((key.to_string(), value.trim().to_string()));
    }
    Ok(())
}

pub(crate) fn apply_limit_to_response(mut value: Value, limit: Option<u32>) -> Value {
    let Some(limit) = limit else {
        return value;
    };
    let limit = limit as usize;

    if let Some(Value::Array(items)) = value.get_mut("data") {
        items.truncate(limit);
    } else if let Some(items) = value.as_array_mut() {
        items.truncate(limit);
    }

    value
}
