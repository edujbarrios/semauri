fn html_escape(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}

fn render_html(artifact: &Artifact) -> Result<String> {
    let Artifact::Web(program) = artifact else {
        return Err(SemauriError::backend(
            "S401",
            "HTML backend cannot render this semantic IR",
            None,
        ));
    };
    let title = html_escape(&program.title);
    let mut body = Vec::new();
    for element in &program.elements {
        let style = if let Some(ValueData::Color(color) | ValueData::String(color)) =
            element.properties.get("color")
        {
            format!(" style=\"color: {}\"", html_escape(color))
        } else {
            String::new()
        };
        let label = html_escape(&element.label);
        let rendered = match element.kind.as_str() {
            "button" => format!("<button{style}>{label}</button>"),
            "image" => format!(
                "<figure{style} data-semauri-kind=\"image\" aria-label=\"{label}\">{label}</figure>"
            ),
            kind => {
                return Err(SemauriError::backend(
                    "S403",
                    format!("HTML backend cannot render element kind '{kind}'"),
                    None,
                ))
            }
        };
        body.push(format!("    {rendered}"));
    }
    let body_text = if body.is_empty() {
        String::new()
    } else {
        format!("\n{}", body.join("\n"))
    };
    Ok(format!(
        "<!doctype html>\n<html lang=\"en\">\n<head>\n  <meta charset=\"utf-8\">\n  <meta name=\"viewport\" content=\"width=device-width, initial-scale=1\">\n  <title>{title}</title>\n</head>\n<body>\n  <h1>{title}</h1>{body_text}\n</body>\n</html>\n"
    ))
}

