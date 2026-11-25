pub struct Names {
    pub snake: String,
    pub pascal: String,
    pub kebab: String,
}

impl Names {
    pub fn from_snake(snake: &str) -> Self {
        let pascal = snake
            .split('_')
            .map(|part| {
                let mut chars = part.chars();
                chars.next().map_or_else(String::new, |first| {
                    first.to_ascii_uppercase().to_string() + chars.as_str()
                })
            })
            .collect();
        Self {
            snake: snake.to_owned(),
            pascal,
            kebab: snake.replace('_', "-"),
        }
    }
}

pub fn domain_mod() -> String {
    "mod errors;\nmod model;\nmod service;\n\npub use model::*;\npub use service::*;\n".to_owned()
}

pub fn domain_model(names: &Names) -> String {
    format!(
        "use ferrodock_core::Id;\nuse serde::Serialize;\n\n#[derive(Debug, Clone, Serialize)]\n\
         pub struct {p} {{\n    pub id: Id,\n    pub name: String,\n}}\n",
        p = names.pascal
    )
}

pub fn domain_service(names: &Names) -> String {
    format!(
        "use ferrodock_core::AppResult;\n\nuse super::model::{p};\n\n#[derive(Debug, Clone, Default)]\n\
         pub struct {p}Service;\n\nimpl {p}Service {{\n    pub async fn list(&self) -> AppResult<Vec<{p}>> {{\n\
         \x20       Ok(Vec::new())\n    }}\n}}\n",
        p = names.pascal
    )
}

pub fn domain_errors(names: &Names) -> String {
    format!(
        "pub const NOT_FOUND: &str = \"{u}_NOT_FOUND\";\n",
        u = names.snake.to_ascii_uppercase()
    )
}

pub fn api_route(names: &Names) -> String {
    format!(
        "use axum::Json;\n\nuse crate::error::ApiResult;\nuse crate::extract::Auth;\n\n\
         #[utoipa::path(\n    get,\n    path = \"/api/v1/{k}\",\n    tag = \"{k}\",\n\
         \x20   description = \"Lists {k} in the active organization\",\n    responses((status = 200))\n)]\n\
         pub async fn list(Auth(_principal): Auth) -> ApiResult<Json<Vec<String>>> {{\n    Ok(Json(Vec::new()))\n}}\n",
        k = names.kebab
    )
}

pub fn migration(names: &Names) -> String {
    format!(
        "CREATE TABLE {s} (\n    id uuid PRIMARY KEY,\n    organization_id uuid NOT NULL REFERENCES organizations (id) ON DELETE CASCADE,\n\
         \x20   name text NOT NULL,\n    created_at timestamptz NOT NULL DEFAULT now()\n);\n",
        s = names.snake
    )
}

pub fn web_api(names: &Names) -> String {
    format!(
        "import {{ queryOptions }} from \"@tanstack/react-query\";\n\nexport const {s}Query = queryOptions({{\n\
         \x20 queryKey: [\"{k}\"],\n  queryFn: async (): Promise<string[]> => [],\n}});\n",
        s = lower_camel(names),
        k = names.kebab
    )
}

pub fn web_page(names: &Names) -> String {
    format!(
        "import {{ useTranslation }} from \"react-i18next\";\n\nexport function {p}Page() {{\n\
         \x20 const {{ t }} = useTranslation(\"{s}\");\n  return <h1 className=\"text-xl text-text\">{{t(\"title\")}}</h1>;\n}}\n",
        p = names.pascal,
        s = names.snake
    )
}

pub fn web_index(names: &Names) -> String {
    format!(
        "export {{ {c}Query }} from \"./api\";\nexport {{ {p}Page }} from \"./{p}Page\";\n",
        c = lower_camel(names),
        p = names.pascal
    )
}

pub fn locale(title: &str) -> String {
    format!("{{\n  \"title\": \"{title}\"\n}}\n")
}

fn lower_camel(names: &Names) -> String {
    let mut chars = names.pascal.chars();
    chars.next().map_or_else(String::new, |first| {
        first.to_ascii_lowercase().to_string() + chars.as_str()
    })
}
