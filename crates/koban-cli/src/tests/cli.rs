use super::*;

#[test]
fn completion_shell_display_uses_documented_names() {
    assert_eq!(CompletionShell::Bash.to_string(), "bash");
    assert_eq!(CompletionShell::Elvish.to_string(), "elvish");
    assert_eq!(CompletionShell::Fish.to_string(), "fish");
    assert_eq!(CompletionShell::Nushell.to_string(), "nushell");
    assert_eq!(CompletionShell::PowerShell.to_string(), "powershell");
    assert_eq!(CompletionShell::Zsh.to_string(), "zsh");
}

#[test]
fn http_methods_render_uppercase_labels() {
    assert_eq!(HttpMethod::Get.label(), "GET");
    assert_eq!(HttpMethod::Post.label(), "POST");
    assert_eq!(HttpMethod::Put.label(), "PUT");
    assert_eq!(HttpMethod::Delete.label(), "DELETE");
}
