#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Resource {
    Clients,
    Invoices,
    Payments,
    Quotes,
    Credits,
    Vendors,
    Expenses,
    Projects,
    Tasks,
}

impl Resource {
    pub fn path(self) -> &'static str {
        match self {
            Self::Clients => "clients",
            Self::Invoices => "invoices",
            Self::Payments => "payments",
            Self::Quotes => "quotes",
            Self::Credits => "credits",
            Self::Vendors => "vendors",
            Self::Expenses => "expenses",
            Self::Projects => "projects",
            Self::Tasks => "tasks",
        }
    }
}
