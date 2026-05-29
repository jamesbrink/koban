use thiserror::Error;

/// Errors returned by the koban library.
///
/// When the `miette` feature is enabled, the variants also carry diagnostic
/// help text via [`miette::Diagnostic`].
#[derive(Debug, Error)]
#[cfg_attr(feature = "miette", derive(miette::Diagnostic))]
pub enum KobanError {
    #[error("Invoice Ninja API token is not configured")]
    #[cfg_attr(
        feature = "miette",
        diagnostic(help(
            "Run `koban auth login` to store a token, or set INVOICE_NINJA_API_TOKEN in your shell. Koban will not call Invoice Ninja without an explicit token."
        ))
    )]
    MissingToken,

    #[error("Invoice Ninja base URL is not valid: {value}")]
    #[cfg_attr(
        feature = "miette",
        diagnostic(help(
            "Set INVOICE_NINJA_BASE_URL to a full URL such as https://invoicing.co or your self-hosted Invoice Ninja domain."
        ))
    )]
    InvalidBaseUrl {
        value: String,
        #[source]
        source: url::ParseError,
    },

    #[error("Invoice Ninja base URL must use HTTPS: {value}")]
    #[cfg_attr(
        feature = "miette",
        diagnostic(help(
            "Use an HTTPS Invoice Ninja URL. Plain HTTP is allowed only for localhost mock tests."
        ))
    )]
    InsecureBaseUrl { value: String },

    #[error("could not build Invoice Ninja API URL for {path}")]
    InvalidEndpoint {
        path: String,
        #[source]
        source: url::ParseError,
    },

    #[error("could not reach Invoice Ninja: {message}")]
    #[cfg_attr(
        feature = "miette",
        diagnostic(help(
            "Check INVOICE_NINJA_BASE_URL, your network, and whether the Invoice Ninja API is reachable."
        ))
    )]
    Transport { message: String },

    #[error("Invoice Ninja returned HTTP {status} for {endpoint}: {message}")]
    Api {
        status: u16,
        endpoint: String,
        message: String,
    },

    #[error("Invoice Ninja returned a response Koban could not decode: {message}")]
    Decode { message: String },

    #[error("list filter is not valid: {value}")]
    #[cfg_attr(
        feature = "miette",
        diagnostic(help("Use key=value, for example `--filter balance=gt:1000`."))
    )]
    InvalidFilter { value: String },

    #[error("invoice payload is not valid: {message}")]
    #[cfg_attr(
        feature = "miette",
        diagnostic(help(
            "Use one raw JSON source (--data, --data-file, or --stdin), or build a payload with guided flags such as --client-id and --line-item."
        ))
    )]
    InvalidPayload { message: String },

    #[error("confirmation required for {operation}")]
    #[cfg_attr(
        feature = "miette",
        diagnostic(help(
            "Review the command with --dry-run, then rerun with --yes when you intentionally want to perform this mutation."
        ))
    )]
    ConfirmationRequired { operation: String },

    #[error("could not write download file: {message}")]
    #[cfg_attr(
        feature = "miette",
        diagnostic(help(
            "Choose a path in an existing directory. Use --force if you intentionally want to overwrite an existing file."
        ))
    )]
    File { message: String },

    #[error("update failed: {message}")]
    #[cfg_attr(
        feature = "miette",
        diagnostic(help(
            "Run `koban update --check` to inspect the latest release without modifying the installed binary."
        ))
    )]
    Update { message: String },

    #[error("could not access stored credentials: {message}")]
    #[cfg_attr(
        feature = "miette",
        diagnostic(help(
            "Check the koban config directory (override it with KOBAN_CONFIG_DIR) and, when using --keychain, that your OS keychain is unlocked and reachable."
        ))
    )]
    Credential { message: String },
}

/// Convenience alias for results returned by the koban library.
pub type Result<T> = std::result::Result<T, KobanError>;
