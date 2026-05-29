use koban::{KobanError, Resource, Result};

use crate::cli::HttpMethod;

#[derive(Debug, Clone, Copy)]
pub(crate) enum ResourceCapability {
    List,
    Show,
    Template,
    EditTemplate,
    Create,
    Update,
    Delete,
    Bulk,
    Upload,
}

impl ResourceCapability {
    fn label(self) -> &'static str {
        match self {
            Self::List => "list",
            Self::Show => "show",
            Self::Template => "template",
            Self::EditTemplate => "edit-template",
            Self::Create => "create",
            Self::Update => "update",
            Self::Delete => "delete",
            Self::Bulk => "bulk",
            Self::Upload => "upload",
        }
    }
}

pub(crate) struct ActionRoute {
    pub(crate) method: HttpMethod,
    pub(crate) path: String,
    pub(crate) body: bool,
    pub(crate) is_bulk: bool,
}

pub(crate) fn require_resource_capability(
    resource: Resource,
    capability: ResourceCapability,
) -> Result<()> {
    if resource_supports(resource, capability) {
        return Ok(());
    }

    Err(KobanError::InvalidPayload {
        message: format!(
            "{} does not support `{}` in the official API",
            resource.label(),
            capability.label()
        ),
    })
}

pub(crate) fn resource_update_path(resource: Resource, id: &str) -> String {
    match resource {
        Resource::PurchaseOrders => format!("api/v1/purchase_order/{id}"),
        _ => format!("api/v1/{}/{id}", resource.path()),
    }
}

pub(crate) fn resource_delete_path(resource: Resource, id: &str) -> String {
    match resource {
        Resource::PurchaseOrders => format!("api/v1/purchase_order/{id}"),
        _ => format!("api/v1/{}/{id}", resource.path()),
    }
}

pub(crate) fn resource_action_route(resource: Resource, id: &str, action: &str) -> ActionRoute {
    match resource {
        Resource::Clients if action == "updateTaxData" => ActionRoute {
            method: HttpMethod::Post,
            path: format!("api/v1/clients/{id}/updateTaxData"),
            body: true,
            is_bulk: false,
        },
        Resource::Payments
        | Resource::Quotes
        | Resource::RecurringInvoices
        | Resource::RecurringQuotes
        | Resource::PurchaseOrders => ActionRoute {
            method: HttpMethod::Get,
            path: format!("api/v1/{}/{id}/{action}", resource.path()),
            body: false,
            is_bulk: false,
        },
        _ => ActionRoute {
            method: HttpMethod::Post,
            path: format!("api/v1/{}/bulk", resource.path()),
            body: true,
            is_bulk: true,
        },
    }
}

pub(crate) fn resource_download_base_path(resource: Resource) -> Option<&'static str> {
    match resource {
        Resource::Quotes => Some("api/v1/quote"),
        Resource::Credits => Some("api/v1/credit"),
        Resource::RecurringInvoices => Some("api/v1/recurring_invoice"),
        Resource::PurchaseOrders => Some("api/v1/purchase_order"),
        _ => None,
    }
}

fn resource_supports(resource: Resource, capability: ResourceCapability) -> bool {
    match capability {
        ResourceCapability::List => {
            !matches!(resource, Resource::CompanyUsers | Resource::Templates)
        }
        ResourceCapability::Show => !matches!(
            resource,
            Resource::Activities
                | Resource::CompanyLedger
                | Resource::CompanyUsers
                | Resource::Documents
                | Resource::Templates
        ),
        ResourceCapability::Template => !matches!(
            resource,
            Resource::Activities
                | Resource::CompanyLedger
                | Resource::CompanyUsers
                | Resource::Documents
                | Resource::TaxRates
                | Resource::Templates
        ),
        ResourceCapability::EditTemplate => !matches!(
            resource,
            Resource::Activities
                | Resource::CompanyLedger
                | Resource::CompanyUsers
                | Resource::Documents
                | Resource::Locations
                | Resource::TaskSchedulers
                | Resource::Templates
        ),
        ResourceCapability::Create => !matches!(
            resource,
            Resource::Activities
                | Resource::CompanyLedger
                | Resource::Documents
                | Resource::TaxRates
        ),
        ResourceCapability::Update | ResourceCapability::Delete => !matches!(
            resource,
            Resource::Activities
                | Resource::CompanyLedger
                | Resource::CompanyUsers
                | Resource::Documents
                | Resource::Templates
        ),
        ResourceCapability::Bulk => !matches!(
            resource,
            Resource::Activities
                | Resource::ClientGatewayTokens
                | Resource::Companies
                | Resource::CompanyLedger
                | Resource::CompanyUsers
                | Resource::Documents
                | Resource::Locations
                | Resource::Templates
        ),
        ResourceCapability::Upload => matches!(
            resource,
            Resource::Clients
                | Resource::Companies
                | Resource::Credits
                | Resource::Expenses
                | Resource::GroupSettings
                | Resource::Invoices
                | Resource::Payments
                | Resource::Products
                | Resource::Projects
                | Resource::PurchaseOrders
                | Resource::Quotes
                | Resource::RecurringExpenses
                | Resource::RecurringInvoices
                | Resource::Tasks
                | Resource::Vendors
        ),
    }
}
