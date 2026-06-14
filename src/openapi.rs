use axum::{ routing::get, Router };

pub fn openapi_router() -> Router {
    Router::new().route("/api-docs/openapi.json", get(openapi_json)).route("/docs", get(swagger_ui))
}

async fn openapi_json() -> impl axum::response::IntoResponse {
    axum::Json(
        serde_json::json!({
        "openapi": "3.0.3",
        "info": {
            "title": "Crypto Payment Gateway API",
            "version": "1.0.0",
            "description": "Production-grade crypto payment infrastructure"
        },
        "servers": [{ "url": "/api/v1" }],
        "paths": {
            "/merchants": {
                "post": {
                    "tags": ["merchants"],
                    "summary": "Create merchant",
                    "requestBody": {
                        "required": true,
                        "content": { "application/json": { "schema": {
                            "type": "object",
                            "properties": {
                                "name": { "type": "string" },
                                "email": { "type": "string", "format": "email" }
                            },
                            "required": ["name", "email"]
                        }}}
                    },
                    "responses": { "201": { "description": "Merchant created" } }
                }
            },
            "/merchants/{id}": {
                "get": {
                    "tags": ["merchants"],
                    "summary": "Get merchant by ID",
                    "parameters": [{ "name": "id", "in": "path", "required": true, "schema": { "type": "string", "format": "uuid" } }],
                    "responses": { "200": { "description": "Merchant found" }, "404": { "description": "Not found" } }
                }
            },
            "/wallets": {
                "post": {
                    "tags": ["wallets"],
                    "summary": "Create wallet",
                    "security": [{ "ApiKeyAuth": [] }],
                    "requestBody": {
                        "required": true,
                        "content": { "application/json": { "schema": {
                            "type": "object",
                            "properties": {
                                "merchant_id": { "type": "string", "format": "uuid" },
                                "address": { "type": "string" }
                            },
                            "required": ["merchant_id", "address"]
                        }}}
                    },
                    "responses": { "201": { "description": "Wallet created" } }
                }
            },
            "/invoices": {
                "post": {
                    "tags": ["invoices"],
                    "summary": "Create invoice",
                    "security": [{ "ApiKeyAuth": [] }],
                    "requestBody": {
                        "required": true,
                        "content": { "application/json": { "schema": {
                            "type": "object",
                            "properties": {
                                "merchant_id": { "type": "string", "format": "uuid" },
                                "wallet_id": { "type": "string", "format": "uuid" },
                                "amount": { "type": "string", "example": "100.00" },
                                "description": { "type": "string" },
                                "expires_at": { "type": "string", "format": "date-time" }
                            },
                            "required": ["merchant_id", "wallet_id", "amount", "expires_at"]
                        }}}
                    },
                    "responses": { "201": { "description": "Invoice created" } }
                }
            },
            "/invoices/{id}": {
                "get": {
                    "tags": ["invoices"],
                    "summary": "Get invoice by ID",
                    "security": [{ "ApiKeyAuth": [] }],
                    "parameters": [{ "name": "id", "in": "path", "required": true, "schema": { "type": "string", "format": "uuid" } }],
                    "responses": { "200": { "description": "Invoice found" }, "404": { "description": "Not found" } }
                }
            },
            "/webhooks": {
                "post": {
                    "tags": ["webhooks"],
                    "summary": "Register webhook endpoint",
                    "security": [{ "ApiKeyAuth": [] }],
                    "requestBody": {
                        "required": true,
                        "content": { "application/json": { "schema": {
                            "type": "object",
                            "properties": {
                                "merchant_id": { "type": "string", "format": "uuid" },
                                "url": { "type": "string", "format": "uri" },
                                "secret": { "type": "string" }
                            },
                            "required": ["merchant_id", "url", "secret"]
                        }}}
                    },
                    "responses": { "201": { "description": "Webhook registered" } }
                }
            }
        },
        "components": {
            "securitySchemes": {
                "ApiKeyAuth": {
                    "type": "apiKey",
                    "in": "header",
                    "name": "x-api-key"
                }
            }
        }
    })
    )
}

async fn swagger_ui() -> impl axum::response::IntoResponse {
    axum::response::Html(
        r#"<!DOCTYPE html>
<html>
<head>
    <title>Crypto Payment Gateway API</title>
    <meta charset="utf-8"/>
    <meta name="viewport" content="width=device-width, initial-scale=1">
    <link rel="stylesheet" type="text/css" href="https://unpkg.com/swagger-ui-dist@5/swagger-ui.css">
</head>
<body>
<div id="swagger-ui"></div>
<script src="https://unpkg.com/swagger-ui-dist@5/swagger-ui-bundle.js"></script>
<script>
    SwaggerUIBundle({
        url: "/api-docs/openapi.json",
        dom_id: '#swagger-ui',
        presets: [SwaggerUIBundle.presets.apis, SwaggerUIBundle.SwaggerUIStandalonePreset],
        layout: "BaseLayout"
    })
</script>
</body>
</html>"#
    )
}
