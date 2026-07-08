
    const schema = {
  "asyncapi": "3.0.0",
  "channels": {
    "client_commands": {
      "address": "/api/ws",
      "x-parser-unique-object-id": "client_commands"
    },
    "server_responses": {
      "address": "/api/ws",
      "x-parser-unique-object-id": "server_responses"
    }
  },
  "components": {
    "messages": {
      "auth.login": {
        "contentType": "application/json",
        "name": "auth.login",
        "payload": {
          "description": "All client-to-server messages",
          "oneOf": [
            {
              "description": "Authentication login request",
              "properties": {
                "client_version": {
                  "description": "Client version",
                  "type": [
                    "string",
                    "null"
                  ],
                  "x-parser-schema-id": "<anonymous-schema-2>"
                },
                "device_id": {
                  "description": "Device ID (from machine-id)",
                  "type": "string",
                  "x-parser-schema-id": "<anonymous-schema-3>"
                },
                "public_key": {
                  "description": "Device public key for encryption",
                  "type": "string",
                  "x-parser-schema-id": "<anonymous-schema-4>"
                },
                "session_id": {
                  "description": "Optional session ID for reconnection",
                  "type": [
                    "string",
                    "null"
                  ],
                  "x-parser-schema-id": "<anonymous-schema-5>"
                }
              },
              "required": [
                "device_id",
                "public_key"
              ],
              "type": "object",
              "x-parser-schema-id": "AuthLoginRequest"
            },
            {
              "description": "Logout request",
              "properties": {
                "session_id": {
                  "description": "Session ID to logout",
                  "type": "string",
                  "x-parser-schema-id": "<anonymous-schema-6>"
                }
              },
              "required": [
                "session_id"
              ],
              "type": "object",
              "x-parser-schema-id": "AuthLogoutRequest"
            },
            {
              "description": "Hosting initialization request",
              "properties": {
                "db_provider": {
                  "description": "Database provider (GCE, GCP_CLOUDSQL, SUPABASE)",
                  "type": "string",
                  "x-parser-schema-id": "<anonymous-schema-7>"
                },
                "dns_provider": {
                  "description": "DNS provider (CLOUDFLARE_DNS, PORKBUN, DUCKDNS, GCP_CLOUDDNS)",
                  "type": "string",
                  "x-parser-schema-id": "<anonymous-schema-8>"
                },
                "dns_provider_token": {
                  "description": "DNS provider token/API key",
                  "type": "string",
                  "x-parser-schema-id": "<anonymous-schema-9>"
                },
                "domain": {
                  "description": "Domain name",
                  "type": "string",
                  "x-parser-schema-id": "<anonymous-schema-10>"
                },
                "web_provider": {
                  "description": "Web provider (GCE, VPS, CLOUDFLARE_PAGES, FIREBASE_HOSTING)",
                  "type": "string",
                  "x-parser-schema-id": "<anonymous-schema-11>"
                },
                "web_provider_token": {
                  "description": "Web provider token/API key",
                  "type": [
                    "string",
                    "null"
                  ],
                  "x-parser-schema-id": "<anonymous-schema-12>"
                }
              },
              "required": [
                "domain",
                "dns_provider",
                "dns_provider_token",
                "web_provider",
                "db_provider"
              ],
              "type": "object",
              "x-parser-schema-id": "HostingInitRequest"
            },
            {
              "description": "Show hosting details request",
              "properties": {
                "hosting_id": {
                  "description": "Optional hosting domain/ID to show (if empty, shows current selected)",
                  "type": [
                    "string",
                    "null"
                  ],
                  "x-parser-schema-id": "<anonymous-schema-13>"
                }
              },
              "type": "object",
              "x-parser-schema-id": "HostingShowRequest"
            },
            {
              "description": "Select hosting request",
              "properties": {
                "hosting_id": {
                  "description": "Hosting ID/domain to select",
                  "type": "string",
                  "x-parser-schema-id": "<anonymous-schema-14>"
                }
              },
              "required": [
                "hosting_id"
              ],
              "type": "object",
              "x-parser-schema-id": "HostingSelectRequest"
            },
            {
              "description": "List hostings request",
              "properties": {
                "filter_active": {
                  "description": "Optional filter by status",
                  "type": [
                    "boolean",
                    "null"
                  ],
                  "x-parser-schema-id": "<anonymous-schema-15>"
                }
              },
              "type": "object",
              "x-parser-schema-id": "HostingListRequest"
            },
            {
              "description": "Close hosting request",
              "properties": {
                "confirm": {
                  "description": "Confirmation flag",
                  "type": "boolean",
                  "x-parser-schema-id": "<anonymous-schema-16>"
                },
                "hosting_id": {
                  "description": "Hosting ID/domain to close",
                  "type": "string",
                  "x-parser-schema-id": "<anonymous-schema-17>"
                }
              },
              "required": [
                "hosting_id",
                "confirm"
              ],
              "type": "object",
              "x-parser-schema-id": "HostingCloseRequest"
            },
            {
              "description": "List members request",
              "properties": {
                "limit": {
                  "description": "Maximum number of members to return",
                  "format": "uint",
                  "minimum": 0,
                  "type": [
                    "integer",
                    "null"
                  ],
                  "x-parser-schema-id": "<anonymous-schema-18>"
                },
                "offset": {
                  "description": "Offset for pagination",
                  "format": "uint",
                  "minimum": 0,
                  "type": [
                    "integer",
                    "null"
                  ],
                  "x-parser-schema-id": "<anonymous-schema-19>"
                },
                "server_id": {
                  "description": "Server/hosting ID",
                  "type": "string",
                  "x-parser-schema-id": "<anonymous-schema-20>"
                }
              },
              "required": [
                "server_id"
              ],
              "type": "object",
              "x-parser-schema-id": "MemberListRequest"
            },
            {
              "description": "Get member info request",
              "properties": {
                "member_id": {
                  "description": "Member ID or user mention (@username)",
                  "type": "string",
                  "x-parser-schema-id": "<anonymous-schema-21>"
                },
                "server_id": {
                  "description": "Server/hosting ID",
                  "type": "string",
                  "x-parser-schema-id": "<anonymous-schema-22>"
                }
              },
              "required": [
                "server_id",
                "member_id"
              ],
              "type": "object",
              "x-parser-schema-id": "MemberInfoRequest"
            },
            {
              "description": "Kick member request",
              "properties": {
                "member_id": {
                  "description": "Member ID to kick",
                  "type": "string",
                  "x-parser-schema-id": "<anonymous-schema-23>"
                },
                "reason": {
                  "description": "Reason for kicking",
                  "type": [
                    "string",
                    "null"
                  ],
                  "x-parser-schema-id": "<anonymous-schema-24>"
                },
                "server_id": {
                  "description": "Server/hosting ID",
                  "type": "string",
                  "x-parser-schema-id": "<anonymous-schema-25>"
                }
              },
              "required": [
                "server_id",
                "member_id"
              ],
              "type": "object",
              "x-parser-schema-id": "MemberKickRequest"
            },
            {
              "description": "Ban member request",
              "properties": {
                "duration_secs": {
                  "description": "Ban duration in seconds (None = permanent)",
                  "format": "uint64",
                  "minimum": 0,
                  "type": [
                    "integer",
                    "null"
                  ],
                  "x-parser-schema-id": "<anonymous-schema-26>"
                },
                "member_id": {
                  "description": "Member ID to ban",
                  "type": "string",
                  "x-parser-schema-id": "<anonymous-schema-27>"
                },
                "reason": {
                  "description": "Reason for banning",
                  "type": [
                    "string",
                    "null"
                  ],
                  "x-parser-schema-id": "<anonymous-schema-28>"
                },
                "server_id": {
                  "description": "Server/hosting ID",
                  "type": "string",
                  "x-parser-schema-id": "<anonymous-schema-29>"
                }
              },
              "required": [
                "server_id",
                "member_id"
              ],
              "type": "object",
              "x-parser-schema-id": "MemberBanRequest"
            },
            {
              "description": "List channels request",
              "properties": {
                "server_id": {
                  "description": "Optional server ID filter",
                  "type": [
                    "string",
                    "null"
                  ],
                  "x-parser-schema-id": "<anonymous-schema-30>"
                }
              },
              "type": "object",
              "x-parser-schema-id": "ChannelListRequest"
            },
            {
              "description": "Get channel info request",
              "properties": {
                "channel_id": {
                  "description": "Channel ID or name (#channel)",
                  "type": "string",
                  "x-parser-schema-id": "<anonymous-schema-31>"
                }
              },
              "required": [
                "channel_id"
              ],
              "type": "object",
              "x-parser-schema-id": "ChannelInfoRequest"
            },
            {
              "description": "Create channel request",
              "properties": {
                "channel_type": {
                  "description": "Channel type",
                  "oneOf": [
                    {
                      "const": "text",
                      "description": "Text channel",
                      "type": "string",
                      "x-parser-schema-id": "<anonymous-schema-32>"
                    },
                    {
                      "const": "voice",
                      "description": "Voice channel",
                      "type": "string",
                      "x-parser-schema-id": "<anonymous-schema-33>"
                    },
                    {
                      "const": "forum",
                      "description": "Forum channel",
                      "type": "string",
                      "x-parser-schema-id": "<anonymous-schema-34>"
                    },
                    {
                      "const": "order",
                      "description": "Order channel (auto-created for orders)",
                      "type": "string",
                      "x-parser-schema-id": "<anonymous-schema-35>"
                    }
                  ],
                  "x-parser-schema-id": "ChannelType"
                },
                "name": {
                  "description": "Channel name",
                  "type": "string",
                  "x-parser-schema-id": "<anonymous-schema-36>"
                },
                "server_id": {
                  "description": "Server ID",
                  "type": "string",
                  "x-parser-schema-id": "<anonymous-schema-37>"
                },
                "slowmode_secs": {
                  "description": "Optional slowmode delay",
                  "format": "uint32",
                  "minimum": 0,
                  "type": [
                    "integer",
                    "null"
                  ],
                  "x-parser-schema-id": "<anonymous-schema-38>"
                },
                "topic": {
                  "description": "Optional topic",
                  "type": [
                    "string",
                    "null"
                  ],
                  "x-parser-schema-id": "<anonymous-schema-39>"
                }
              },
              "required": [
                "server_id",
                "name",
                "channel_type"
              ],
              "type": "object",
              "x-parser-schema-id": "ChannelCreateRequest"
            },
            {
              "description": "Edit channel request",
              "properties": {
                "channel_id": {
                  "description": "Channel ID to edit",
                  "type": "string",
                  "x-parser-schema-id": "<anonymous-schema-40>"
                },
                "name": {
                  "description": "New name (optional)",
                  "type": [
                    "string",
                    "null"
                  ],
                  "x-parser-schema-id": "<anonymous-schema-41>"
                },
                "slowmode_secs": {
                  "description": "New slowmode delay (optional)",
                  "format": "uint32",
                  "minimum": 0,
                  "type": [
                    "integer",
                    "null"
                  ],
                  "x-parser-schema-id": "<anonymous-schema-42>"
                },
                "topic": {
                  "description": "New topic (optional)",
                  "type": [
                    "string",
                    "null"
                  ],
                  "x-parser-schema-id": "<anonymous-schema-43>"
                }
              },
              "required": [
                "channel_id"
              ],
              "type": "object",
              "x-parser-schema-id": "ChannelEditRequest"
            },
            {
              "description": "Delete channel request",
              "properties": {
                "channel_id": {
                  "description": "Channel ID to delete",
                  "type": "string",
                  "x-parser-schema-id": "<anonymous-schema-44>"
                },
                "confirm": {
                  "description": "Confirmation flag",
                  "type": "boolean",
                  "x-parser-schema-id": "<anonymous-schema-45>"
                }
              },
              "required": [
                "channel_id",
                "confirm"
              ],
              "type": "object",
              "x-parser-schema-id": "ChannelDeleteRequest"
            },
            {
              "description": "Send message request",
              "properties": {
                "attachments": {
                  "description": "Optional file attachments (base64 encoded)",
                  "items": {
                    "description": "Message attachment",
                    "properties": {
                      "content_type": {
                        "description": "MIME type",
                        "type": "string",
                        "x-parser-schema-id": "<anonymous-schema-47>"
                      },
                      "data": {
                        "description": "File data (base64 encoded)",
                        "type": "string",
                        "x-parser-schema-id": "<anonymous-schema-48>"
                      },
                      "filename": {
                        "description": "File name",
                        "type": "string",
                        "x-parser-schema-id": "<anonymous-schema-49>"
                      },
                      "size": {
                        "description": "File size in bytes",
                        "format": "uint",
                        "minimum": 0,
                        "type": "integer",
                        "x-parser-schema-id": "<anonymous-schema-50>"
                      }
                    },
                    "required": [
                      "filename",
                      "content_type",
                      "data",
                      "size"
                    ],
                    "type": "object",
                    "x-parser-schema-id": "MessageAttachment"
                  },
                  "type": [
                    "array",
                    "null"
                  ],
                  "x-parser-schema-id": "<anonymous-schema-46>"
                },
                "channel_id": {
                  "description": "Channel ID to send to",
                  "type": "string",
                  "x-parser-schema-id": "<anonymous-schema-51>"
                },
                "content": {
                  "description": "Message content",
                  "type": "string",
                  "x-parser-schema-id": "<anonymous-schema-52>"
                },
                "reply_to": {
                  "description": "Optional reply to message ID",
                  "type": [
                    "string",
                    "null"
                  ],
                  "x-parser-schema-id": "<anonymous-schema-53>"
                }
              },
              "required": [
                "channel_id",
                "content"
              ],
              "type": "object",
              "x-parser-schema-id": "MessageSendRequest"
            },
            {
              "description": "List messages request",
              "properties": {
                "after": {
                  "description": "Message ID to start after (for pagination)",
                  "type": [
                    "string",
                    "null"
                  ],
                  "x-parser-schema-id": "<anonymous-schema-54>"
                },
                "before": {
                  "description": "Message ID to start before (for pagination)",
                  "type": [
                    "string",
                    "null"
                  ],
                  "x-parser-schema-id": "<anonymous-schema-55>"
                },
                "channel_id": {
                  "description": "Channel ID",
                  "type": "string",
                  "x-parser-schema-id": "<anonymous-schema-56>"
                },
                "limit": {
                  "description": "Number of messages to retrieve (default: 10)",
                  "format": "uint",
                  "minimum": 0,
                  "type": [
                    "integer",
                    "null"
                  ],
                  "x-parser-schema-id": "<anonymous-schema-57>"
                }
              },
              "required": [
                "channel_id"
              ],
              "type": "object",
              "x-parser-schema-id": "MessageListRequest"
            },
            {
              "description": "Edit message request",
              "properties": {
                "channel_id": {
                  "description": "Channel ID",
                  "type": "string",
                  "x-parser-schema-id": "<anonymous-schema-58>"
                },
                "content": {
                  "description": "New content",
                  "type": "string",
                  "x-parser-schema-id": "<anonymous-schema-59>"
                },
                "message_id": {
                  "description": "Message ID to edit",
                  "type": "string",
                  "x-parser-schema-id": "<anonymous-schema-60>"
                }
              },
              "required": [
                "channel_id",
                "message_id",
                "content"
              ],
              "type": "object",
              "x-parser-schema-id": "MessageEditRequest"
            },
            {
              "description": "Delete message request",
              "properties": {
                "channel_id": {
                  "description": "Channel ID",
                  "type": "string",
                  "x-parser-schema-id": "<anonymous-schema-61>"
                },
                "confirm": {
                  "description": "Confirmation flag",
                  "type": "boolean",
                  "x-parser-schema-id": "<anonymous-schema-62>"
                },
                "message_id": {
                  "description": "Message ID to delete",
                  "type": "string",
                  "x-parser-schema-id": "<anonymous-schema-63>"
                }
              },
              "required": [
                "channel_id",
                "message_id",
                "confirm"
              ],
              "type": "object",
              "x-parser-schema-id": "MessageDeleteRequest"
            },
            {
              "description": "Reply to message request",
              "properties": {
                "attachments": {
                  "description": "Optional attachments",
                  "items": "$ref:$.components.messages.auth.login.payload.oneOf[16].properties.attachments.items",
                  "type": [
                    "array",
                    "null"
                  ],
                  "x-parser-schema-id": "<anonymous-schema-64>"
                },
                "channel_id": {
                  "description": "Channel ID",
                  "type": "string",
                  "x-parser-schema-id": "<anonymous-schema-65>"
                },
                "content": {
                  "description": "Reply content",
                  "type": "string",
                  "x-parser-schema-id": "<anonymous-schema-66>"
                },
                "message_id": {
                  "description": "Message ID to reply to",
                  "type": "string",
                  "x-parser-schema-id": "<anonymous-schema-67>"
                }
              },
              "required": [
                "channel_id",
                "message_id",
                "content"
              ],
              "type": "object",
              "x-parser-schema-id": "MessageReplyRequest"
            },
            {
              "description": "Create product request",
              "properties": {
                "category": {
                  "description": "Product category",
                  "type": "string",
                  "x-parser-schema-id": "<anonymous-schema-68>"
                },
                "contents": {
                  "description": "Product description/contents",
                  "type": "string",
                  "x-parser-schema-id": "<anonymous-schema-69>"
                },
                "image": {
                  "description": "Product image URL or base64 data",
                  "type": "string",
                  "x-parser-schema-id": "<anonymous-schema-70>"
                },
                "name": {
                  "description": "Product name",
                  "type": "string",
                  "x-parser-schema-id": "<anonymous-schema-71>"
                },
                "price": {
                  "description": "Product price information",
                  "properties": {
                    "amount": {
                      "description": "Price amount",
                      "format": "double",
                      "type": "number",
                      "x-parser-schema-id": "<anonymous-schema-72>"
                    },
                    "currency": {
                      "description": "Currency code (USD, KRW, etc.)",
                      "type": "string",
                      "x-parser-schema-id": "<anonymous-schema-73>"
                    },
                    "discount_percent": {
                      "description": "Optional discount percentage",
                      "format": "float",
                      "type": [
                        "number",
                        "null"
                      ],
                      "x-parser-schema-id": "<anonymous-schema-74>"
                    }
                  },
                  "required": [
                    "amount",
                    "currency"
                  ],
                  "type": "object",
                  "x-parser-schema-id": "ProductPrice"
                },
                "server_id": {
                  "description": "Server ID",
                  "type": "string",
                  "x-parser-schema-id": "<anonymous-schema-75>"
                },
                "sku": {
                  "description": "Product SKU",
                  "type": [
                    "string",
                    "null"
                  ],
                  "x-parser-schema-id": "<anonymous-schema-76>"
                },
                "stock": {
                  "description": "Product stock quantity",
                  "format": "uint32",
                  "minimum": 0,
                  "type": "integer",
                  "x-parser-schema-id": "<anonymous-schema-77>"
                }
              },
              "required": [
                "server_id",
                "name",
                "category",
                "image",
                "contents",
                "price",
                "stock"
              ],
              "type": "object",
              "x-parser-schema-id": "ProductCreateRequest"
            },
            {
              "description": "List products request",
              "properties": {
                "category": {
                  "description": "Optional category filter",
                  "type": [
                    "string",
                    "null"
                  ],
                  "x-parser-schema-id": "<anonymous-schema-78>"
                },
                "limit": {
                  "description": "Limit",
                  "format": "uint",
                  "minimum": 0,
                  "type": [
                    "integer",
                    "null"
                  ],
                  "x-parser-schema-id": "<anonymous-schema-79>"
                },
                "offset": {
                  "description": "Offset for pagination",
                  "format": "uint",
                  "minimum": 0,
                  "type": [
                    "integer",
                    "null"
                  ],
                  "x-parser-schema-id": "<anonymous-schema-80>"
                },
                "search": {
                  "description": "Optional search query",
                  "type": [
                    "string",
                    "null"
                  ],
                  "x-parser-schema-id": "<anonymous-schema-81>"
                },
                "server_id": {
                  "description": "Server ID",
                  "type": "string",
                  "x-parser-schema-id": "<anonymous-schema-82>"
                }
              },
              "required": [
                "server_id"
              ],
              "type": "object",
              "x-parser-schema-id": "ProductListRequest"
            },
            {
              "description": "Modify product request",
              "properties": {
                "category": {
                  "description": "New category (optional)",
                  "type": [
                    "string",
                    "null"
                  ],
                  "x-parser-schema-id": "<anonymous-schema-83>"
                },
                "contents": {
                  "description": "New description (optional)",
                  "type": [
                    "string",
                    "null"
                  ],
                  "x-parser-schema-id": "<anonymous-schema-84>"
                },
                "image": {
                  "description": "New image (optional)",
                  "type": [
                    "string",
                    "null"
                  ],
                  "x-parser-schema-id": "<anonymous-schema-85>"
                },
                "is_available": {
                  "description": "New availability (optional)",
                  "type": [
                    "boolean",
                    "null"
                  ],
                  "x-parser-schema-id": "<anonymous-schema-86>"
                },
                "name": {
                  "description": "New name (optional)",
                  "type": [
                    "string",
                    "null"
                  ],
                  "x-parser-schema-id": "<anonymous-schema-87>"
                },
                "price": {
                  "anyOf": [
                    "$ref:$.components.messages.auth.login.payload.oneOf[21].properties.price",
                    {
                      "type": "null",
                      "x-parser-schema-id": "<anonymous-schema-89>"
                    }
                  ],
                  "description": "New price (optional)",
                  "x-parser-schema-id": "<anonymous-schema-88>"
                },
                "product_id": {
                  "description": "Product ID to modify",
                  "type": "string",
                  "x-parser-schema-id": "<anonymous-schema-90>"
                },
                "server_id": {
                  "description": "Server ID",
                  "type": "string",
                  "x-parser-schema-id": "<anonymous-schema-91>"
                },
                "stock": {
                  "description": "New stock (optional)",
                  "format": "uint32",
                  "minimum": 0,
                  "type": [
                    "integer",
                    "null"
                  ],
                  "x-parser-schema-id": "<anonymous-schema-92>"
                }
              },
              "required": [
                "server_id",
                "product_id"
              ],
              "type": "object",
              "x-parser-schema-id": "ProductModifyRequest"
            },
            {
              "description": "Delete product request",
              "properties": {
                "confirm": {
                  "description": "Confirmation flag",
                  "type": "boolean",
                  "x-parser-schema-id": "<anonymous-schema-93>"
                },
                "product_id": {
                  "description": "Product ID to delete",
                  "type": "string",
                  "x-parser-schema-id": "<anonymous-schema-94>"
                },
                "server_id": {
                  "description": "Server ID",
                  "type": "string",
                  "x-parser-schema-id": "<anonymous-schema-95>"
                }
              },
              "required": [
                "server_id",
                "product_id",
                "confirm"
              ],
              "type": "object",
              "x-parser-schema-id": "ProductDeleteRequest"
            },
            {
              "description": "Create order request",
              "properties": {
                "counts": {
                  "description": "Quantities for each product (parallel array)",
                  "items": {
                    "format": "uint32",
                    "minimum": 0,
                    "type": "integer",
                    "x-parser-schema-id": "<anonymous-schema-97>"
                  },
                  "type": "array",
                  "x-parser-schema-id": "<anonymous-schema-96>"
                },
                "notes": {
                  "description": "Optional order notes",
                  "type": [
                    "string",
                    "null"
                  ],
                  "x-parser-schema-id": "<anonymous-schema-98>"
                },
                "product_ids": {
                  "description": "List of product IDs",
                  "items": {
                    "type": "string",
                    "x-parser-schema-id": "<anonymous-schema-100>"
                  },
                  "type": "array",
                  "x-parser-schema-id": "<anonymous-schema-99>"
                },
                "server_id": {
                  "description": "Server/store ID to order from",
                  "type": "string",
                  "x-parser-schema-id": "<anonymous-schema-101>"
                },
                "shipping_address": {
                  "description": "Shipping address",
                  "properties": {
                    "address_line1": {
                      "description": "Address line 1",
                      "type": "string",
                      "x-parser-schema-id": "<anonymous-schema-102>"
                    },
                    "address_line2": {
                      "description": "Address line 2",
                      "type": [
                        "string",
                        "null"
                      ],
                      "x-parser-schema-id": "<anonymous-schema-103>"
                    },
                    "city": {
                      "description": "City",
                      "type": "string",
                      "x-parser-schema-id": "<anonymous-schema-104>"
                    },
                    "country": {
                      "description": "Country code (ISO 3166-1 alpha-2)",
                      "type": "string",
                      "x-parser-schema-id": "<anonymous-schema-105>"
                    },
                    "phone": {
                      "description": "Phone number",
                      "type": "string",
                      "x-parser-schema-id": "<anonymous-schema-106>"
                    },
                    "postal_code": {
                      "description": "Postal/ZIP code",
                      "type": "string",
                      "x-parser-schema-id": "<anonymous-schema-107>"
                    },
                    "recipient_name": {
                      "description": "Recipient name",
                      "type": "string",
                      "x-parser-schema-id": "<anonymous-schema-108>"
                    },
                    "state": {
                      "description": "State/Province",
                      "type": [
                        "string",
                        "null"
                      ],
                      "x-parser-schema-id": "<anonymous-schema-109>"
                    }
                  },
                  "required": [
                    "recipient_name",
                    "phone",
                    "address_line1",
                    "city",
                    "postal_code",
                    "country"
                  ],
                  "type": "object",
                  "x-parser-schema-id": "ShippingAddress"
                }
              },
              "required": [
                "server_id",
                "product_ids",
                "counts",
                "shipping_address"
              ],
              "type": "object",
              "x-parser-schema-id": "OrderCreateRequest"
            },
            {
              "description": "List orders request",
              "properties": {
                "limit": {
                  "description": "Limit",
                  "format": "uint",
                  "minimum": 0,
                  "type": [
                    "integer",
                    "null"
                  ],
                  "x-parser-schema-id": "<anonymous-schema-110>"
                },
                "offset": {
                  "description": "Offset for pagination",
                  "format": "uint",
                  "minimum": 0,
                  "type": [
                    "integer",
                    "null"
                  ],
                  "x-parser-schema-id": "<anonymous-schema-111>"
                },
                "server_id": {
                  "description": "Server ID (optional - if empty, lists all orders)",
                  "type": [
                    "string",
                    "null"
                  ],
                  "x-parser-schema-id": "<anonymous-schema-112>"
                },
                "status": {
                  "anyOf": [
                    {
                      "description": "Order status",
                      "oneOf": [
                        {
                          "const": "pending",
                          "description": "Order pending approval",
                          "type": "string",
                          "x-parser-schema-id": "<anonymous-schema-114>"
                        },
                        {
                          "const": "processing",
                          "description": "Order processing",
                          "type": "string",
                          "x-parser-schema-id": "<anonymous-schema-115>"
                        },
                        {
                          "const": "paid",
                          "description": "Payment completed",
                          "type": "string",
                          "x-parser-schema-id": "<anonymous-schema-116>"
                        },
                        {
                          "const": "shipped",
                          "description": "Order shipped",
                          "type": "string",
                          "x-parser-schema-id": "<anonymous-schema-117>"
                        },
                        {
                          "const": "delivered",
                          "description": "Order delivered",
                          "type": "string",
                          "x-parser-schema-id": "<anonymous-schema-118>"
                        },
                        {
                          "const": "cancelled",
                          "description": "Order cancelled",
                          "type": "string",
                          "x-parser-schema-id": "<anonymous-schema-119>"
                        },
                        {
                          "const": "refunded",
                          "description": "Order refunded",
                          "type": "string",
                          "x-parser-schema-id": "<anonymous-schema-120>"
                        }
                      ],
                      "x-parser-schema-id": "OrderStatus"
                    },
                    {
                      "type": "null",
                      "x-parser-schema-id": "<anonymous-schema-121>"
                    }
                  ],
                  "description": "Filter by status (optional)",
                  "x-parser-schema-id": "<anonymous-schema-113>"
                }
              },
              "type": "object",
              "x-parser-schema-id": "OrderListRequest"
            },
            {
              "description": "Create payment request",
              "properties": {
                "order_id": {
                  "description": "Order ID to pay for",
                  "type": "string",
                  "x-parser-schema-id": "<anonymous-schema-122>"
                },
                "payment_method": {
                  "description": "Payment method",
                  "oneOf": [
                    {
                      "additionalProperties": false,
                      "description": "Portone payment gateway",
                      "properties": {
                        "portone": {
                          "properties": {
                            "merchant_id": {
                              "description": "Portone merchant ID",
                              "type": "string",
                              "x-parser-schema-id": "<anonymous-schema-125>"
                            }
                          },
                          "required": [
                            "merchant_id"
                          ],
                          "type": "object",
                          "x-parser-schema-id": "<anonymous-schema-124>"
                        }
                      },
                      "required": [
                        "portone"
                      ],
                      "type": "object",
                      "x-parser-schema-id": "<anonymous-schema-123>"
                    },
                    {
                      "additionalProperties": false,
                      "description": "KakaoPay",
                      "properties": {
                        "kakao_pay": {
                          "properties": {
                            "merchant_id": {
                              "description": "KakaoPay merchant ID",
                              "type": "string",
                              "x-parser-schema-id": "<anonymous-schema-128>"
                            }
                          },
                          "required": [
                            "merchant_id"
                          ],
                          "type": "object",
                          "x-parser-schema-id": "<anonymous-schema-127>"
                        }
                      },
                      "required": [
                        "kakao_pay"
                      ],
                      "type": "object",
                      "x-parser-schema-id": "<anonymous-schema-126>"
                    },
                    {
                      "additionalProperties": false,
                      "description": "Credit card",
                      "properties": {
                        "credit_card": {
                          "properties": {
                            "card_token": {
                              "description": "Card token (PCI compliant)",
                              "type": "string",
                              "x-parser-schema-id": "<anonymous-schema-131>"
                            }
                          },
                          "required": [
                            "card_token"
                          ],
                          "type": "object",
                          "x-parser-schema-id": "<anonymous-schema-130>"
                        }
                      },
                      "required": [
                        "credit_card"
                      ],
                      "type": "object",
                      "x-parser-schema-id": "<anonymous-schema-129>"
                    },
                    {
                      "additionalProperties": false,
                      "description": "Bank transfer",
                      "properties": {
                        "bank_transfer": {
                          "properties": {
                            "bank_code": {
                              "description": "Bank code",
                              "type": "string",
                              "x-parser-schema-id": "<anonymous-schema-134>"
                            }
                          },
                          "required": [
                            "bank_code"
                          ],
                          "type": "object",
                          "x-parser-schema-id": "<anonymous-schema-133>"
                        }
                      },
                      "required": [
                        "bank_transfer"
                      ],
                      "type": "object",
                      "x-parser-schema-id": "<anonymous-schema-132>"
                    }
                  ],
                  "x-parser-schema-id": "PaymentMethod"
                },
                "return_url": {
                  "description": "Return URL after payment",
                  "type": [
                    "string",
                    "null"
                  ],
                  "x-parser-schema-id": "<anonymous-schema-135>"
                },
                "server_id": {
                  "description": "Server ID",
                  "type": "string",
                  "x-parser-schema-id": "<anonymous-schema-136>"
                }
              },
              "required": [
                "server_id",
                "order_id",
                "payment_method"
              ],
              "type": "object",
              "x-parser-schema-id": "PaymentCreateRequest"
            },
            {
              "description": "Verify payment request (from payment gateway webhook)",
              "properties": {
                "payment_id": {
                  "description": "Payment ID",
                  "type": "string",
                  "x-parser-schema-id": "<anonymous-schema-137>"
                },
                "server_id": {
                  "description": "Server ID",
                  "type": "string",
                  "x-parser-schema-id": "<anonymous-schema-138>"
                },
                "verification_data": {
                  "description": "Payment gateway verification data (JSON)",
                  "x-parser-schema-id": "<anonymous-schema-139>"
                }
              },
              "required": [
                "server_id",
                "payment_id",
                "verification_data"
              ],
              "type": "object",
              "x-parser-schema-id": "PaymentVerifyRequest"
            },
            {
              "description": "List payments request",
              "properties": {
                "limit": {
                  "description": "Limit",
                  "format": "uint",
                  "minimum": 0,
                  "type": [
                    "integer",
                    "null"
                  ],
                  "x-parser-schema-id": "<anonymous-schema-140>"
                },
                "offset": {
                  "description": "Offset for pagination",
                  "format": "uint",
                  "minimum": 0,
                  "type": [
                    "integer",
                    "null"
                  ],
                  "x-parser-schema-id": "<anonymous-schema-141>"
                },
                "order_id": {
                  "description": "Filter by order ID (optional)",
                  "type": [
                    "string",
                    "null"
                  ],
                  "x-parser-schema-id": "<anonymous-schema-142>"
                },
                "server_id": {
                  "description": "Server ID (optional)",
                  "type": [
                    "string",
                    "null"
                  ],
                  "x-parser-schema-id": "<anonymous-schema-143>"
                },
                "status": {
                  "anyOf": [
                    {
                      "description": "Payment status",
                      "oneOf": [
                        {
                          "const": "pending",
                          "description": "Payment pending",
                          "type": "string",
                          "x-parser-schema-id": "<anonymous-schema-145>"
                        },
                        {
                          "const": "processing",
                          "description": "Payment processing",
                          "type": "string",
                          "x-parser-schema-id": "<anonymous-schema-146>"
                        },
                        {
                          "const": "completed",
                          "description": "Payment completed",
                          "type": "string",
                          "x-parser-schema-id": "<anonymous-schema-147>"
                        },
                        {
                          "const": "failed",
                          "description": "Payment failed",
                          "type": "string",
                          "x-parser-schema-id": "<anonymous-schema-148>"
                        },
                        {
                          "const": "cancelled",
                          "description": "Payment cancelled",
                          "type": "string",
                          "x-parser-schema-id": "<anonymous-schema-149>"
                        },
                        {
                          "const": "refunded",
                          "description": "Payment refunded",
                          "type": "string",
                          "x-parser-schema-id": "<anonymous-schema-150>"
                        }
                      ],
                      "x-parser-schema-id": "PaymentStatus"
                    },
                    {
                      "type": "null",
                      "x-parser-schema-id": "<anonymous-schema-151>"
                    }
                  ],
                  "description": "Filter by status (optional)",
                  "x-parser-schema-id": "<anonymous-schema-144>"
                }
              },
              "type": "object",
              "x-parser-schema-id": "PaymentListRequest"
            },
            {
              "description": "Create review request",
              "properties": {
                "content": {
                  "description": "Review content",
                  "type": "string",
                  "x-parser-schema-id": "<anonymous-schema-152>"
                },
                "images": {
                  "description": "Optional images (base64 encoded)",
                  "items": {
                    "description": "Review image",
                    "properties": {
                      "content_type": {
                        "description": "Image MIME type",
                        "type": "string",
                        "x-parser-schema-id": "<anonymous-schema-154>"
                      },
                      "data": {
                        "description": "Image data (base64 encoded)",
                        "type": "string",
                        "x-parser-schema-id": "<anonymous-schema-155>"
                      },
                      "filename": {
                        "description": "Image filename",
                        "type": "string",
                        "x-parser-schema-id": "<anonymous-schema-156>"
                      }
                    },
                    "required": [
                      "filename",
                      "data",
                      "content_type"
                    ],
                    "type": "object",
                    "x-parser-schema-id": "ReviewImage"
                  },
                  "type": [
                    "array",
                    "null"
                  ],
                  "x-parser-schema-id": "<anonymous-schema-153>"
                },
                "order_id": {
                  "description": "Order ID this review is for",
                  "type": "string",
                  "x-parser-schema-id": "<anonymous-schema-157>"
                },
                "product_id": {
                  "description": "Product ID being reviewed (optional, if specific product)",
                  "type": [
                    "string",
                    "null"
                  ],
                  "x-parser-schema-id": "<anonymous-schema-158>"
                },
                "rating": {
                  "description": "Rating (1-5 stars)",
                  "format": "uint8",
                  "maximum": 255,
                  "minimum": 0,
                  "type": "integer",
                  "x-parser-schema-id": "<anonymous-schema-159>"
                },
                "server_id": {
                  "description": "Server/store ID being reviewed",
                  "type": "string",
                  "x-parser-schema-id": "<anonymous-schema-160>"
                },
                "title": {
                  "description": "Review title",
                  "type": "string",
                  "x-parser-schema-id": "<anonymous-schema-161>"
                }
              },
              "required": [
                "server_id",
                "order_id",
                "rating",
                "title",
                "content"
              ],
              "type": "object",
              "x-parser-schema-id": "ReviewCreateRequest"
            },
            {
              "description": "List reviews request",
              "properties": {
                "limit": {
                  "description": "Limit",
                  "format": "uint",
                  "minimum": 0,
                  "type": [
                    "integer",
                    "null"
                  ],
                  "x-parser-schema-id": "<anonymous-schema-162>"
                },
                "min_rating": {
                  "description": "Filter by minimum rating (optional)",
                  "format": "uint8",
                  "maximum": 255,
                  "minimum": 0,
                  "type": [
                    "integer",
                    "null"
                  ],
                  "x-parser-schema-id": "<anonymous-schema-163>"
                },
                "offset": {
                  "description": "Offset for pagination",
                  "format": "uint",
                  "minimum": 0,
                  "type": [
                    "integer",
                    "null"
                  ],
                  "x-parser-schema-id": "<anonymous-schema-164>"
                },
                "product_id": {
                  "description": "Filter by product ID (optional)",
                  "type": [
                    "string",
                    "null"
                  ],
                  "x-parser-schema-id": "<anonymous-schema-165>"
                },
                "server_id": {
                  "description": "Server ID to list reviews for",
                  "type": "string",
                  "x-parser-schema-id": "<anonymous-schema-166>"
                },
                "sort_by": {
                  "anyOf": [
                    {
                      "description": "Review sort options",
                      "oneOf": [
                        {
                          "const": "recent",
                          "description": "Most recent first",
                          "type": "string",
                          "x-parser-schema-id": "<anonymous-schema-168>"
                        },
                        {
                          "const": "highest_rated",
                          "description": "Highest rated first",
                          "type": "string",
                          "x-parser-schema-id": "<anonymous-schema-169>"
                        },
                        {
                          "const": "lowest_rated",
                          "description": "Lowest rated first",
                          "type": "string",
                          "x-parser-schema-id": "<anonymous-schema-170>"
                        },
                        {
                          "const": "most_helpful",
                          "description": "Most helpful first",
                          "type": "string",
                          "x-parser-schema-id": "<anonymous-schema-171>"
                        }
                      ],
                      "x-parser-schema-id": "ReviewSortBy"
                    },
                    {
                      "type": "null",
                      "x-parser-schema-id": "<anonymous-schema-172>"
                    }
                  ],
                  "description": "Sort order",
                  "x-parser-schema-id": "<anonymous-schema-167>"
                }
              },
              "required": [
                "server_id"
              ],
              "type": "object",
              "x-parser-schema-id": "ReviewListRequest"
            }
          ],
          "title": "ClientMessage",
          "x-parser-schema-id": "<anonymous-schema-1>"
        },
        "title": "auth.login",
        "x-parser-unique-object-id": "auth.login"
      },
      "auth.logout": {
        "contentType": "application/json",
        "name": "auth.logout",
        "payload": {
          "description": "All client-to-server messages",
          "oneOf": [
            "$ref:$.components.messages.auth.login.payload.oneOf[0]",
            "$ref:$.components.messages.auth.login.payload.oneOf[1]",
            "$ref:$.components.messages.auth.login.payload.oneOf[2]",
            "$ref:$.components.messages.auth.login.payload.oneOf[3]",
            "$ref:$.components.messages.auth.login.payload.oneOf[4]",
            "$ref:$.components.messages.auth.login.payload.oneOf[5]",
            "$ref:$.components.messages.auth.login.payload.oneOf[6]",
            "$ref:$.components.messages.auth.login.payload.oneOf[7]",
            "$ref:$.components.messages.auth.login.payload.oneOf[8]",
            "$ref:$.components.messages.auth.login.payload.oneOf[9]",
            "$ref:$.components.messages.auth.login.payload.oneOf[10]",
            "$ref:$.components.messages.auth.login.payload.oneOf[11]",
            "$ref:$.components.messages.auth.login.payload.oneOf[12]",
            "$ref:$.components.messages.auth.login.payload.oneOf[13]",
            "$ref:$.components.messages.auth.login.payload.oneOf[14]",
            "$ref:$.components.messages.auth.login.payload.oneOf[15]",
            "$ref:$.components.messages.auth.login.payload.oneOf[16]",
            "$ref:$.components.messages.auth.login.payload.oneOf[17]",
            "$ref:$.components.messages.auth.login.payload.oneOf[18]",
            "$ref:$.components.messages.auth.login.payload.oneOf[19]",
            "$ref:$.components.messages.auth.login.payload.oneOf[20]",
            "$ref:$.components.messages.auth.login.payload.oneOf[21]",
            "$ref:$.components.messages.auth.login.payload.oneOf[22]",
            "$ref:$.components.messages.auth.login.payload.oneOf[23]",
            "$ref:$.components.messages.auth.login.payload.oneOf[24]",
            "$ref:$.components.messages.auth.login.payload.oneOf[25]",
            "$ref:$.components.messages.auth.login.payload.oneOf[26]",
            "$ref:$.components.messages.auth.login.payload.oneOf[27]",
            "$ref:$.components.messages.auth.login.payload.oneOf[28]",
            "$ref:$.components.messages.auth.login.payload.oneOf[29]",
            "$ref:$.components.messages.auth.login.payload.oneOf[30]",
            "$ref:$.components.messages.auth.login.payload.oneOf[31]"
          ],
          "title": "ClientMessage",
          "x-parser-schema-id": "<anonymous-schema-173>"
        },
        "title": "auth.logout",
        "x-parser-unique-object-id": "auth.logout"
      },
      "auth.logout.response": {
        "contentType": "application/json",
        "name": "auth.logout.response",
        "payload": {
          "description": "All server-to-client messages",
          "oneOf": [
            {
              "description": "Authentication response",
              "properties": {
                "device_info": {
                  "anyOf": [
                    {
                      "description": "Device information",
                      "properties": {
                        "device_id": {
                          "description": "Device ID",
                          "type": "string",
                          "x-parser-schema-id": "<anonymous-schema-176>"
                        },
                        "device_name": {
                          "description": "Device name/hostname",
                          "type": [
                            "string",
                            "null"
                          ],
                          "x-parser-schema-id": "<anonymous-schema-177>"
                        },
                        "last_seen": {
                          "description": "Last seen timestamp",
                          "format": "date-time",
                          "type": [
                            "string",
                            "null"
                          ],
                          "x-parser-schema-id": "<anonymous-schema-178>"
                        },
                        "platform": {
                          "description": "Device platform (linux, windows, macos, android)",
                          "type": [
                            "string",
                            "null"
                          ],
                          "x-parser-schema-id": "<anonymous-schema-179>"
                        }
                      },
                      "required": [
                        "device_id"
                      ],
                      "type": "object",
                      "x-parser-schema-id": "DeviceInfo"
                    },
                    {
                      "type": "null",
                      "x-parser-schema-id": "<anonymous-schema-180>"
                    }
                  ],
                  "description": "Authenticated user/device information",
                  "x-parser-schema-id": "<anonymous-schema-175>"
                },
                "error": {
                  "description": "Error message if authentication failed",
                  "type": [
                    "string",
                    "null"
                  ],
                  "x-parser-schema-id": "<anonymous-schema-181>"
                },
                "expires_at": {
                  "description": "Session expiry time",
                  "format": "date-time",
                  "type": [
                    "string",
                    "null"
                  ],
                  "x-parser-schema-id": "<anonymous-schema-182>"
                },
                "server_public_key": {
                  "description": "Server public key for encryption",
                  "type": [
                    "string",
                    "null"
                  ],
                  "x-parser-schema-id": "<anonymous-schema-183>"
                },
                "session_id": {
                  "description": "Session ID for this connection",
                  "type": [
                    "string",
                    "null"
                  ],
                  "x-parser-schema-id": "<anonymous-schema-184>"
                },
                "success": {
                  "description": "Whether authentication was successful",
                  "type": "boolean",
                  "x-parser-schema-id": "<anonymous-schema-185>"
                }
              },
              "required": [
                "success"
              ],
              "type": "object",
              "x-parser-schema-id": "AuthResponse"
            },
            {
              "description": "Logout response",
              "properties": {
                "message": {
                  "description": "Optional message",
                  "type": [
                    "string",
                    "null"
                  ],
                  "x-parser-schema-id": "<anonymous-schema-186>"
                },
                "success": {
                  "description": "Whether logout was successful",
                  "type": "boolean",
                  "x-parser-schema-id": "<anonymous-schema-187>"
                }
              },
              "required": [
                "success"
              ],
              "type": "object",
              "x-parser-schema-id": "AuthLogoutResponse"
            },
            {
              "description": "Hosting initialization response",
              "properties": {
                "domain": {
                  "description": "Domain name",
                  "type": [
                    "string",
                    "null"
                  ],
                  "x-parser-schema-id": "<anonymous-schema-188>"
                },
                "error": {
                  "description": "Error message if failed",
                  "type": [
                    "string",
                    "null"
                  ],
                  "x-parser-schema-id": "<anonymous-schema-189>"
                },
                "hosting_id": {
                  "description": "Hosting ID",
                  "type": [
                    "string",
                    "null"
                  ],
                  "x-parser-schema-id": "<anonymous-schema-190>"
                },
                "steps_completed": {
                  "description": "Setup steps completed",
                  "items": {
                    "type": "string",
                    "x-parser-schema-id": "<anonymous-schema-192>"
                  },
                  "type": "array",
                  "x-parser-schema-id": "<anonymous-schema-191>"
                },
                "success": {
                  "description": "Whether initialization was successful",
                  "type": "boolean",
                  "x-parser-schema-id": "<anonymous-schema-193>"
                }
              },
              "required": [
                "success",
                "steps_completed"
              ],
              "type": "object",
              "x-parser-schema-id": "HostingInitResponse"
            },
            {
              "description": "Show hosting details response",
              "properties": {
                "error": {
                  "description": "Error message if not found",
                  "type": [
                    "string",
                    "null"
                  ],
                  "x-parser-schema-id": "<anonymous-schema-194>"
                },
                "hosting": {
                  "anyOf": [
                    {
                      "description": "Hosting details",
                      "properties": {
                        "db_provider": {
                          "description": "Database provider",
                          "type": "string",
                          "x-parser-schema-id": "<anonymous-schema-196>"
                        },
                        "dns_provider": {
                          "description": "DNS provider",
                          "type": "string",
                          "x-parser-schema-id": "<anonymous-schema-197>"
                        },
                        "domain": {
                          "description": "Domain name",
                          "type": "string",
                          "x-parser-schema-id": "<anonymous-schema-198>"
                        },
                        "id": {
                          "description": "Hosting ID",
                          "type": "string",
                          "x-parser-schema-id": "<anonymous-schema-199>"
                        },
                        "is_active": {
                          "description": "Whether hosting is active",
                          "type": "boolean",
                          "x-parser-schema-id": "<anonymous-schema-200>"
                        },
                        "is_selected": {
                          "description": "Whether hosting is selected for operations",
                          "type": "boolean",
                          "x-parser-schema-id": "<anonymous-schema-201>"
                        },
                        "server_ip": {
                          "description": "Server IP address",
                          "type": [
                            "string",
                            "null"
                          ],
                          "x-parser-schema-id": "<anonymous-schema-202>"
                        },
                        "ssl_status": {
                          "description": "SSL certificate status",
                          "type": [
                            "string",
                            "null"
                          ],
                          "x-parser-schema-id": "<anonymous-schema-203>"
                        },
                        "web_provider": {
                          "description": "Web provider",
                          "type": "string",
                          "x-parser-schema-id": "<anonymous-schema-204>"
                        }
                      },
                      "required": [
                        "id",
                        "domain",
                        "dns_provider",
                        "web_provider",
                        "db_provider",
                        "is_active",
                        "is_selected"
                      ],
                      "type": "object",
                      "x-parser-schema-id": "HostingDetails"
                    },
                    {
                      "type": "null",
                      "x-parser-schema-id": "<anonymous-schema-205>"
                    }
                  ],
                  "description": "Hosting details",
                  "x-parser-schema-id": "<anonymous-schema-195>"
                }
              },
              "type": "object",
              "x-parser-schema-id": "HostingShowResponse"
            },
            {
              "description": "Select hosting response",
              "properties": {
                "error": {
                  "description": "Error message if failed",
                  "type": [
                    "string",
                    "null"
                  ],
                  "x-parser-schema-id": "<anonymous-schema-206>"
                },
                "hosting_id": {
                  "description": "Selected hosting ID",
                  "type": [
                    "string",
                    "null"
                  ],
                  "x-parser-schema-id": "<anonymous-schema-207>"
                },
                "success": {
                  "description": "Whether selection was successful",
                  "type": "boolean",
                  "x-parser-schema-id": "<anonymous-schema-208>"
                }
              },
              "required": [
                "success"
              ],
              "type": "object",
              "x-parser-schema-id": "HostingSelectResponse"
            },
            {
              "description": "List hostings response",
              "properties": {
                "hostings": {
                  "description": "List of hostings",
                  "items": "$ref:$.components.messages.auth.logout.response.payload.oneOf[3].properties.hosting.anyOf[0]",
                  "type": "array",
                  "x-parser-schema-id": "<anonymous-schema-209>"
                },
                "total": {
                  "description": "Total count",
                  "format": "uint",
                  "minimum": 0,
                  "type": "integer",
                  "x-parser-schema-id": "<anonymous-schema-210>"
                }
              },
              "required": [
                "hostings",
                "total"
              ],
              "type": "object",
              "x-parser-schema-id": "HostingListResponse"
            },
            {
              "description": "List members response",
              "properties": {
                "has_more": {
                  "description": "Whether there are more members",
                  "type": "boolean",
                  "x-parser-schema-id": "<anonymous-schema-211>"
                },
                "members": {
                  "description": "List of members",
                  "items": {
                    "description": "Member information",
                    "properties": {
                      "display_name": {
                        "description": "Member display name",
                        "type": [
                          "string",
                          "null"
                        ],
                        "x-parser-schema-id": "<anonymous-schema-213>"
                      },
                      "is_online": {
                        "description": "Whether member is online",
                        "type": "boolean",
                        "x-parser-schema-id": "<anonymous-schema-214>"
                      },
                      "joined_at": {
                        "description": "Join date",
                        "format": "date-time",
                        "type": [
                          "string",
                          "null"
                        ],
                        "x-parser-schema-id": "<anonymous-schema-215>"
                      },
                      "last_seen": {
                        "description": "Last seen timestamp",
                        "format": "date-time",
                        "type": [
                          "string",
                          "null"
                        ],
                        "x-parser-schema-id": "<anonymous-schema-216>"
                      },
                      "member_id": {
                        "description": "Member ID (domain name for stores, device ID for owner devices)",
                        "type": "string",
                        "x-parser-schema-id": "<anonymous-schema-217>"
                      },
                      "member_type": {
                        "description": "Member type",
                        "oneOf": [
                          {
                            "const": "owner",
                            "description": "Owner device member (no dots in ID)",
                            "type": "string",
                            "x-parser-schema-id": "<anonymous-schema-218>"
                          },
                          {
                            "const": "store",
                            "description": "Store member (dots allowed in ID)",
                            "type": "string",
                            "x-parser-schema-id": "<anonymous-schema-219>"
                          }
                        ],
                        "x-parser-schema-id": "MemberType"
                      },
                      "roles": {
                        "description": "Roles assigned to this member",
                        "items": {
                          "type": "string",
                          "x-parser-schema-id": "<anonymous-schema-221>"
                        },
                        "type": "array",
                        "x-parser-schema-id": "<anonymous-schema-220>"
                      }
                    },
                    "required": [
                      "member_id",
                      "member_type",
                      "roles",
                      "is_online"
                    ],
                    "type": "object",
                    "x-parser-schema-id": "MemberInfo"
                  },
                  "type": "array",
                  "x-parser-schema-id": "<anonymous-schema-212>"
                },
                "total": {
                  "description": "Total member count",
                  "format": "uint",
                  "minimum": 0,
                  "type": "integer",
                  "x-parser-schema-id": "<anonymous-schema-222>"
                }
              },
              "required": [
                "members",
                "total",
                "has_more"
              ],
              "type": "object",
              "x-parser-schema-id": "MemberListResponse"
            },
            {
              "description": "Get member info response",
              "properties": {
                "error": {
                  "description": "Error message if not found",
                  "type": [
                    "string",
                    "null"
                  ],
                  "x-parser-schema-id": "<anonymous-schema-223>"
                },
                "member": {
                  "anyOf": [
                    "$ref:$.components.messages.auth.logout.response.payload.oneOf[6].properties.members.items",
                    {
                      "type": "null",
                      "x-parser-schema-id": "<anonymous-schema-225>"
                    }
                  ],
                  "description": "Member information",
                  "x-parser-schema-id": "<anonymous-schema-224>"
                }
              },
              "type": "object",
              "x-parser-schema-id": "MemberInfoResponse"
            },
            {
              "description": "Member kicked notification",
              "properties": {
                "kicked_by": {
                  "description": "Who kicked the member",
                  "type": "string",
                  "x-parser-schema-id": "<anonymous-schema-226>"
                },
                "member_id": {
                  "description": "Kicked member ID",
                  "type": "string",
                  "x-parser-schema-id": "<anonymous-schema-227>"
                },
                "reason": {
                  "description": "Reason",
                  "type": [
                    "string",
                    "null"
                  ],
                  "x-parser-schema-id": "<anonymous-schema-228>"
                },
                "server_id": {
                  "description": "Server ID",
                  "type": "string",
                  "x-parser-schema-id": "<anonymous-schema-229>"
                },
                "timestamp": {
                  "description": "Timestamp",
                  "format": "date-time",
                  "type": "string",
                  "x-parser-schema-id": "<anonymous-schema-230>"
                }
              },
              "required": [
                "server_id",
                "member_id",
                "kicked_by",
                "timestamp"
              ],
              "type": "object",
              "x-parser-schema-id": "MemberKickedNotification"
            },
            {
              "description": "Member banned notification",
              "properties": {
                "banned_by": {
                  "description": "Who banned the member",
                  "type": "string",
                  "x-parser-schema-id": "<anonymous-schema-231>"
                },
                "expires_at": {
                  "description": "Ban expiry time (None = permanent)",
                  "format": "date-time",
                  "type": [
                    "string",
                    "null"
                  ],
                  "x-parser-schema-id": "<anonymous-schema-232>"
                },
                "member_id": {
                  "description": "Banned member ID",
                  "type": "string",
                  "x-parser-schema-id": "<anonymous-schema-233>"
                },
                "reason": {
                  "description": "Reason",
                  "type": [
                    "string",
                    "null"
                  ],
                  "x-parser-schema-id": "<anonymous-schema-234>"
                },
                "server_id": {
                  "description": "Server ID",
                  "type": "string",
                  "x-parser-schema-id": "<anonymous-schema-235>"
                },
                "timestamp": {
                  "description": "Timestamp",
                  "format": "date-time",
                  "type": "string",
                  "x-parser-schema-id": "<anonymous-schema-236>"
                }
              },
              "required": [
                "server_id",
                "member_id",
                "banned_by",
                "timestamp"
              ],
              "type": "object",
              "x-parser-schema-id": "MemberBannedNotification"
            },
            {
              "description": "List channels response",
              "properties": {
                "channels": {
                  "description": "List of channels",
                  "items": {
                    "description": "Channel information",
                    "properties": {
                      "channel_id": {
                        "description": "Channel ID",
                        "type": "string",
                        "x-parser-schema-id": "<anonymous-schema-238>"
                      },
                      "channel_type": "$ref:$.components.messages.auth.login.payload.oneOf[13].properties.channel_type",
                      "created_at": {
                        "description": "Created timestamp",
                        "format": "date-time",
                        "type": [
                          "string",
                          "null"
                        ],
                        "x-parser-schema-id": "<anonymous-schema-239>"
                      },
                      "is_nsfw": {
                        "description": "Whether channel is NSFW",
                        "type": "boolean",
                        "x-parser-schema-id": "<anonymous-schema-240>"
                      },
                      "name": {
                        "description": "Channel name",
                        "type": "string",
                        "x-parser-schema-id": "<anonymous-schema-241>"
                      },
                      "server_id": {
                        "description": "Server ID this channel belongs to",
                        "type": "string",
                        "x-parser-schema-id": "<anonymous-schema-242>"
                      },
                      "slowmode_secs": {
                        "description": "Slowmode delay in seconds",
                        "format": "uint32",
                        "minimum": 0,
                        "type": [
                          "integer",
                          "null"
                        ],
                        "x-parser-schema-id": "<anonymous-schema-243>"
                      },
                      "topic": {
                        "description": "Channel topic/description",
                        "type": [
                          "string",
                          "null"
                        ],
                        "x-parser-schema-id": "<anonymous-schema-244>"
                      }
                    },
                    "required": [
                      "channel_id",
                      "name",
                      "channel_type",
                      "server_id",
                      "is_nsfw"
                    ],
                    "type": "object",
                    "x-parser-schema-id": "ChannelInfo"
                  },
                  "type": "array",
                  "x-parser-schema-id": "<anonymous-schema-237>"
                },
                "total": {
                  "description": "Total count",
                  "format": "uint",
                  "minimum": 0,
                  "type": "integer",
                  "x-parser-schema-id": "<anonymous-schema-245>"
                }
              },
              "required": [
                "channels",
                "total"
              ],
              "type": "object",
              "x-parser-schema-id": "ChannelListResponse"
            },
            {
              "description": "Get channel info response",
              "properties": {
                "channel": {
                  "anyOf": [
                    "$ref:$.components.messages.auth.logout.response.payload.oneOf[10].properties.channels.items",
                    {
                      "type": "null",
                      "x-parser-schema-id": "<anonymous-schema-247>"
                    }
                  ],
                  "description": "Channel information",
                  "x-parser-schema-id": "<anonymous-schema-246>"
                },
                "error": {
                  "description": "Error message if not found",
                  "type": [
                    "string",
                    "null"
                  ],
                  "x-parser-schema-id": "<anonymous-schema-248>"
                }
              },
              "type": "object",
              "x-parser-schema-id": "ChannelInfoResponse"
            },
            {
              "description": "Channel created notification",
              "properties": {
                "channel": "$ref:$.components.messages.auth.logout.response.payload.oneOf[10].properties.channels.items",
                "created_by": {
                  "description": "Who created the channel",
                  "type": "string",
                  "x-parser-schema-id": "<anonymous-schema-249>"
                },
                "timestamp": {
                  "description": "Timestamp",
                  "format": "date-time",
                  "type": "string",
                  "x-parser-schema-id": "<anonymous-schema-250>"
                }
              },
              "required": [
                "channel",
                "created_by",
                "timestamp"
              ],
              "type": "object",
              "x-parser-schema-id": "ChannelCreatedNotification"
            },
            {
              "description": "Channel edited notification",
              "properties": {
                "channel": "$ref:$.components.messages.auth.logout.response.payload.oneOf[10].properties.channels.items",
                "edited_by": {
                  "description": "Who edited the channel",
                  "type": "string",
                  "x-parser-schema-id": "<anonymous-schema-251>"
                },
                "timestamp": {
                  "description": "Timestamp",
                  "format": "date-time",
                  "type": "string",
                  "x-parser-schema-id": "<anonymous-schema-252>"
                }
              },
              "required": [
                "channel",
                "edited_by",
                "timestamp"
              ],
              "type": "object",
              "x-parser-schema-id": "ChannelEditedNotification"
            },
            {
              "description": "Channel deleted notification",
              "properties": {
                "channel_id": {
                  "description": "Deleted channel ID",
                  "type": "string",
                  "x-parser-schema-id": "<anonymous-schema-253>"
                },
                "deleted_by": {
                  "description": "Who deleted the channel",
                  "type": "string",
                  "x-parser-schema-id": "<anonymous-schema-254>"
                },
                "server_id": {
                  "description": "Server ID",
                  "type": "string",
                  "x-parser-schema-id": "<anonymous-schema-255>"
                },
                "timestamp": {
                  "description": "Timestamp",
                  "format": "date-time",
                  "type": "string",
                  "x-parser-schema-id": "<anonymous-schema-256>"
                }
              },
              "required": [
                "channel_id",
                "server_id",
                "deleted_by",
                "timestamp"
              ],
              "type": "object",
              "x-parser-schema-id": "ChannelDeletedNotification"
            },
            {
              "description": "Message sent response",
              "properties": {
                "error": {
                  "description": "Error if failed",
                  "type": [
                    "string",
                    "null"
                  ],
                  "x-parser-schema-id": "<anonymous-schema-257>"
                },
                "message": {
                  "anyOf": [
                    {
                      "description": "Message data",
                      "properties": {
                        "attachments": {
                          "description": "Attachments",
                          "items": "$ref:$.components.messages.auth.login.payload.oneOf[16].properties.attachments.items",
                          "type": [
                            "array",
                            "null"
                          ],
                          "x-parser-schema-id": "<anonymous-schema-259>"
                        },
                        "author_id": {
                          "description": "Author member ID",
                          "type": "string",
                          "x-parser-schema-id": "<anonymous-schema-260>"
                        },
                        "channel_id": {
                          "description": "Channel ID",
                          "type": "string",
                          "x-parser-schema-id": "<anonymous-schema-261>"
                        },
                        "content": {
                          "description": "Message content",
                          "type": "string",
                          "x-parser-schema-id": "<anonymous-schema-262>"
                        },
                        "edited_at": {
                          "description": "Edited timestamp",
                          "format": "date-time",
                          "type": [
                            "string",
                            "null"
                          ],
                          "x-parser-schema-id": "<anonymous-schema-263>"
                        },
                        "message_id": {
                          "description": "Message ID",
                          "type": "string",
                          "x-parser-schema-id": "<anonymous-schema-264>"
                        },
                        "reactions": {
                          "description": "Reactions",
                          "items": {
                            "description": "Message reaction",
                            "properties": {
                              "count": {
                                "description": "Count of reactions",
                                "format": "uint32",
                                "minimum": 0,
                                "type": "integer",
                                "x-parser-schema-id": "<anonymous-schema-266>"
                              },
                              "emoji": {
                                "description": "Emoji",
                                "type": "string",
                                "x-parser-schema-id": "<anonymous-schema-267>"
                              },
                              "me": {
                                "description": "Whether current user reacted",
                                "type": "boolean",
                                "x-parser-schema-id": "<anonymous-schema-268>"
                              }
                            },
                            "required": [
                              "emoji",
                              "count",
                              "me"
                            ],
                            "type": "object",
                            "x-parser-schema-id": "MessageReaction"
                          },
                          "type": [
                            "array",
                            "null"
                          ],
                          "x-parser-schema-id": "<anonymous-schema-265>"
                        },
                        "reply_to": {
                          "description": "Reply to message ID",
                          "type": [
                            "string",
                            "null"
                          ],
                          "x-parser-schema-id": "<anonymous-schema-269>"
                        },
                        "timestamp": {
                          "description": "Timestamp",
                          "format": "date-time",
                          "type": "string",
                          "x-parser-schema-id": "<anonymous-schema-270>"
                        }
                      },
                      "required": [
                        "message_id",
                        "channel_id",
                        "author_id",
                        "content",
                        "timestamp"
                      ],
                      "type": "object",
                      "x-parser-schema-id": "MessageData"
                    },
                    {
                      "type": "null",
                      "x-parser-schema-id": "<anonymous-schema-271>"
                    }
                  ],
                  "description": "Message details",
                  "x-parser-schema-id": "<anonymous-schema-258>"
                },
                "message_id": {
                  "description": "Sent message ID",
                  "type": [
                    "string",
                    "null"
                  ],
                  "x-parser-schema-id": "<anonymous-schema-272>"
                },
                "success": {
                  "description": "Whether send was successful",
                  "type": "boolean",
                  "x-parser-schema-id": "<anonymous-schema-273>"
                }
              },
              "required": [
                "success"
              ],
              "type": "object",
              "x-parser-schema-id": "MessageSentResponse"
            },
            {
              "description": "List messages response",
              "properties": {
                "has_more": {
                  "description": "Whether there are more messages",
                  "type": "boolean",
                  "x-parser-schema-id": "<anonymous-schema-274>"
                },
                "messages": {
                  "description": "List of messages",
                  "items": "$ref:$.components.messages.auth.logout.response.payload.oneOf[15].properties.message.anyOf[0]",
                  "type": "array",
                  "x-parser-schema-id": "<anonymous-schema-275>"
                },
                "total": {
                  "description": "Total count (if available)",
                  "format": "uint",
                  "minimum": 0,
                  "type": [
                    "integer",
                    "null"
                  ],
                  "x-parser-schema-id": "<anonymous-schema-276>"
                }
              },
              "required": [
                "messages",
                "has_more"
              ],
              "type": "object",
              "x-parser-schema-id": "MessageListResponse"
            },
            {
              "description": "Message received notification (broadcast to subscribers)",
              "properties": {
                "message": "$ref:$.components.messages.auth.logout.response.payload.oneOf[15].properties.message.anyOf[0]"
              },
              "required": [
                "message"
              ],
              "type": "object",
              "x-parser-schema-id": "MessageReceivedNotification"
            },
            {
              "description": "Message edited notification",
              "properties": {
                "message": "$ref:$.components.messages.auth.logout.response.payload.oneOf[15].properties.message.anyOf[0]"
              },
              "required": [
                "message"
              ],
              "type": "object",
              "x-parser-schema-id": "MessageEditedNotification"
            },
            {
              "description": "Message deleted notification",
              "properties": {
                "channel_id": {
                  "description": "Channel ID",
                  "type": "string",
                  "x-parser-schema-id": "<anonymous-schema-277>"
                },
                "deleted_by": {
                  "description": "Who deleted the message",
                  "type": "string",
                  "x-parser-schema-id": "<anonymous-schema-278>"
                },
                "message_id": {
                  "description": "Deleted message ID",
                  "type": "string",
                  "x-parser-schema-id": "<anonymous-schema-279>"
                },
                "timestamp": {
                  "description": "Timestamp",
                  "format": "date-time",
                  "type": "string",
                  "x-parser-schema-id": "<anonymous-schema-280>"
                }
              },
              "required": [
                "channel_id",
                "message_id",
                "deleted_by",
                "timestamp"
              ],
              "type": "object",
              "x-parser-schema-id": "MessageDeletedNotification"
            },
            {
              "description": "Product created response",
              "properties": {
                "error": {
                  "description": "Error if failed",
                  "type": [
                    "string",
                    "null"
                  ],
                  "x-parser-schema-id": "<anonymous-schema-281>"
                },
                "product": {
                  "anyOf": [
                    {
                      "description": "Product data",
                      "properties": {
                        "category": {
                          "description": "Product category",
                          "type": "string",
                          "x-parser-schema-id": "<anonymous-schema-283>"
                        },
                        "created_at": {
                          "description": "Created timestamp",
                          "format": "date-time",
                          "type": "string",
                          "x-parser-schema-id": "<anonymous-schema-284>"
                        },
                        "description": {
                          "description": "Product description",
                          "type": "string",
                          "x-parser-schema-id": "<anonymous-schema-285>"
                        },
                        "image_url": {
                          "description": "Product image URL",
                          "type": "string",
                          "x-parser-schema-id": "<anonymous-schema-286>"
                        },
                        "is_available": {
                          "description": "Whether product is available",
                          "type": "boolean",
                          "x-parser-schema-id": "<anonymous-schema-287>"
                        },
                        "name": {
                          "description": "Product name",
                          "type": "string",
                          "x-parser-schema-id": "<anonymous-schema-288>"
                        },
                        "price": "$ref:$.components.messages.auth.login.payload.oneOf[21].properties.price",
                        "product_id": {
                          "description": "Product ID",
                          "type": "string",
                          "x-parser-schema-id": "<anonymous-schema-289>"
                        },
                        "server_id": {
                          "description": "Server ID",
                          "type": "string",
                          "x-parser-schema-id": "<anonymous-schema-290>"
                        },
                        "sku": {
                          "description": "SKU",
                          "type": [
                            "string",
                            "null"
                          ],
                          "x-parser-schema-id": "<anonymous-schema-291>"
                        },
                        "stock": {
                          "description": "Stock quantity",
                          "format": "uint32",
                          "minimum": 0,
                          "type": "integer",
                          "x-parser-schema-id": "<anonymous-schema-292>"
                        },
                        "updated_at": {
                          "description": "Updated timestamp",
                          "format": "date-time",
                          "type": [
                            "string",
                            "null"
                          ],
                          "x-parser-schema-id": "<anonymous-schema-293>"
                        }
                      },
                      "required": [
                        "product_id",
                        "server_id",
                        "name",
                        "category",
                        "image_url",
                        "description",
                        "price",
                        "stock",
                        "created_at",
                        "is_available"
                      ],
                      "type": "object",
                      "x-parser-schema-id": "ProductData"
                    },
                    {
                      "type": "null",
                      "x-parser-schema-id": "<anonymous-schema-294>"
                    }
                  ],
                  "description": "Product details",
                  "x-parser-schema-id": "<anonymous-schema-282>"
                },
                "product_id": {
                  "description": "Created product ID",
                  "type": [
                    "string",
                    "null"
                  ],
                  "x-parser-schema-id": "<anonymous-schema-295>"
                },
                "success": {
                  "description": "Whether creation was successful",
                  "type": "boolean",
                  "x-parser-schema-id": "<anonymous-schema-296>"
                }
              },
              "required": [
                "success"
              ],
              "type": "object",
              "x-parser-schema-id": "ProductCreatedResponse"
            },
            {
              "description": "List products response",
              "properties": {
                "has_more": {
                  "description": "Whether there are more products",
                  "type": "boolean",
                  "x-parser-schema-id": "<anonymous-schema-297>"
                },
                "products": {
                  "description": "List of products",
                  "items": "$ref:$.components.messages.auth.logout.response.payload.oneOf[20].properties.product.anyOf[0]",
                  "type": "array",
                  "x-parser-schema-id": "<anonymous-schema-298>"
                },
                "total": {
                  "description": "Total count",
                  "format": "uint",
                  "minimum": 0,
                  "type": "integer",
                  "x-parser-schema-id": "<anonymous-schema-299>"
                }
              },
              "required": [
                "products",
                "total",
                "has_more"
              ],
              "type": "object",
              "x-parser-schema-id": "ProductListResponse"
            },
            {
              "description": "Product modified notification",
              "properties": {
                "modified_by": {
                  "description": "Who modified the product",
                  "type": "string",
                  "x-parser-schema-id": "<anonymous-schema-300>"
                },
                "product": "$ref:$.components.messages.auth.logout.response.payload.oneOf[20].properties.product.anyOf[0]",
                "timestamp": {
                  "description": "Timestamp",
                  "format": "date-time",
                  "type": "string",
                  "x-parser-schema-id": "<anonymous-schema-301>"
                }
              },
              "required": [
                "product",
                "modified_by",
                "timestamp"
              ],
              "type": "object",
              "x-parser-schema-id": "ProductModifiedNotification"
            },
            {
              "description": "Product deleted notification",
              "properties": {
                "deleted_by": {
                  "description": "Who deleted the product",
                  "type": "string",
                  "x-parser-schema-id": "<anonymous-schema-302>"
                },
                "product_id": {
                  "description": "Deleted product ID",
                  "type": "string",
                  "x-parser-schema-id": "<anonymous-schema-303>"
                },
                "server_id": {
                  "description": "Server ID",
                  "type": "string",
                  "x-parser-schema-id": "<anonymous-schema-304>"
                },
                "timestamp": {
                  "description": "Timestamp",
                  "format": "date-time",
                  "type": "string",
                  "x-parser-schema-id": "<anonymous-schema-305>"
                }
              },
              "required": [
                "product_id",
                "server_id",
                "deleted_by",
                "timestamp"
              ],
              "type": "object",
              "x-parser-schema-id": "ProductDeletedNotification"
            },
            {
              "description": "Order created response",
              "properties": {
                "error": {
                  "description": "Error if failed",
                  "type": [
                    "string",
                    "null"
                  ],
                  "x-parser-schema-id": "<anonymous-schema-306>"
                },
                "order": {
                  "anyOf": [
                    {
                      "description": "Order data",
                      "properties": {
                        "channel_id": {
                          "description": "Channel ID for order communication",
                          "type": [
                            "string",
                            "null"
                          ],
                          "x-parser-schema-id": "<anonymous-schema-308>"
                        },
                        "created_at": {
                          "description": "Created timestamp",
                          "format": "date-time",
                          "type": "string",
                          "x-parser-schema-id": "<anonymous-schema-309>"
                        },
                        "customer_id": {
                          "description": "Customer member ID",
                          "type": "string",
                          "x-parser-schema-id": "<anonymous-schema-310>"
                        },
                        "items": {
                          "description": "Order items",
                          "items": {
                            "description": "Order item",
                            "properties": {
                              "product_id": {
                                "description": "Product ID",
                                "type": "string",
                                "x-parser-schema-id": "<anonymous-schema-312>"
                              },
                              "product_name": {
                                "description": "Product name (snapshot at order time)",
                                "type": "string",
                                "x-parser-schema-id": "<anonymous-schema-313>"
                              },
                              "quantity": {
                                "description": "Quantity ordered",
                                "format": "uint32",
                                "minimum": 0,
                                "type": "integer",
                                "x-parser-schema-id": "<anonymous-schema-314>"
                              },
                              "subtotal": "$ref:$.components.messages.auth.login.payload.oneOf[21].properties.price",
                              "unit_price": "$ref:$.components.messages.auth.login.payload.oneOf[21].properties.price"
                            },
                            "required": [
                              "product_id",
                              "product_name",
                              "quantity",
                              "unit_price",
                              "subtotal"
                            ],
                            "type": "object",
                            "x-parser-schema-id": "OrderItem"
                          },
                          "type": "array",
                          "x-parser-schema-id": "<anonymous-schema-311>"
                        },
                        "notes": {
                          "description": "Order notes",
                          "type": [
                            "string",
                            "null"
                          ],
                          "x-parser-schema-id": "<anonymous-schema-315>"
                        },
                        "order_id": {
                          "description": "Order ID",
                          "type": "string",
                          "x-parser-schema-id": "<anonymous-schema-316>"
                        },
                        "server_id": {
                          "description": "Server/store ID",
                          "type": "string",
                          "x-parser-schema-id": "<anonymous-schema-317>"
                        },
                        "shipping_address": "$ref:$.components.messages.auth.login.payload.oneOf[25].properties.shipping_address",
                        "status": "$ref:$.components.messages.auth.login.payload.oneOf[26].properties.status.anyOf[0]",
                        "total_price": "$ref:$.components.messages.auth.login.payload.oneOf[21].properties.price",
                        "updated_at": {
                          "description": "Updated timestamp",
                          "format": "date-time",
                          "type": [
                            "string",
                            "null"
                          ],
                          "x-parser-schema-id": "<anonymous-schema-318>"
                        }
                      },
                      "required": [
                        "order_id",
                        "server_id",
                        "customer_id",
                        "items",
                        "total_price",
                        "shipping_address",
                        "status",
                        "created_at"
                      ],
                      "type": "object",
                      "x-parser-schema-id": "OrderData"
                    },
                    {
                      "type": "null",
                      "x-parser-schema-id": "<anonymous-schema-319>"
                    }
                  ],
                  "description": "Order details",
                  "x-parser-schema-id": "<anonymous-schema-307>"
                },
                "order_id": {
                  "description": "Created order ID",
                  "type": [
                    "string",
                    "null"
                  ],
                  "x-parser-schema-id": "<anonymous-schema-320>"
                },
                "success": {
                  "description": "Whether creation was successful",
                  "type": "boolean",
                  "x-parser-schema-id": "<anonymous-schema-321>"
                }
              },
              "required": [
                "success"
              ],
              "type": "object",
              "x-parser-schema-id": "OrderCreatedResponse"
            },
            {
              "description": "List orders response",
              "properties": {
                "has_more": {
                  "description": "Whether there are more orders",
                  "type": "boolean",
                  "x-parser-schema-id": "<anonymous-schema-322>"
                },
                "orders": {
                  "description": "List of orders",
                  "items": "$ref:$.components.messages.auth.logout.response.payload.oneOf[24].properties.order.anyOf[0]",
                  "type": "array",
                  "x-parser-schema-id": "<anonymous-schema-323>"
                },
                "total": {
                  "description": "Total count",
                  "format": "uint",
                  "minimum": 0,
                  "type": "integer",
                  "x-parser-schema-id": "<anonymous-schema-324>"
                }
              },
              "required": [
                "orders",
                "total",
                "has_more"
              ],
              "type": "object",
              "x-parser-schema-id": "OrderListResponse"
            },
            {
              "description": "Order status update notification",
              "properties": {
                "message": {
                  "description": "Optional message",
                  "type": [
                    "string",
                    "null"
                  ],
                  "x-parser-schema-id": "<anonymous-schema-325>"
                },
                "order_id": {
                  "description": "Order ID",
                  "type": "string",
                  "x-parser-schema-id": "<anonymous-schema-326>"
                },
                "previous_status": "$ref:$.components.messages.auth.login.payload.oneOf[26].properties.status.anyOf[0]",
                "server_id": {
                  "description": "Server ID",
                  "type": "string",
                  "x-parser-schema-id": "<anonymous-schema-327>"
                },
                "status": "$ref:$.components.messages.auth.login.payload.oneOf[26].properties.status.anyOf[0]",
                "timestamp": {
                  "description": "Timestamp",
                  "format": "date-time",
                  "type": "string",
                  "x-parser-schema-id": "<anonymous-schema-328>"
                },
                "tracking_number": {
                  "description": "Tracking number (if shipped)",
                  "type": [
                    "string",
                    "null"
                  ],
                  "x-parser-schema-id": "<anonymous-schema-329>"
                },
                "updated_by": {
                  "description": "Who updated the status",
                  "type": "string",
                  "x-parser-schema-id": "<anonymous-schema-330>"
                }
              },
              "required": [
                "order_id",
                "server_id",
                "status",
                "previous_status",
                "updated_by",
                "timestamp"
              ],
              "type": "object",
              "x-parser-schema-id": "OrderStatusUpdateNotification"
            },
            {
              "description": "Payment created response",
              "properties": {
                "error": {
                  "description": "Error if failed",
                  "type": [
                    "string",
                    "null"
                  ],
                  "x-parser-schema-id": "<anonymous-schema-331>"
                },
                "payment": {
                  "anyOf": [
                    {
                      "description": "Payment data",
                      "properties": {
                        "amount": "$ref:$.components.messages.auth.login.payload.oneOf[21].properties.price",
                        "completed_at": {
                          "description": "Completed timestamp",
                          "format": "date-time",
                          "type": [
                            "string",
                            "null"
                          ],
                          "x-parser-schema-id": "<anonymous-schema-333>"
                        },
                        "created_at": {
                          "description": "Created timestamp",
                          "format": "date-time",
                          "type": "string",
                          "x-parser-schema-id": "<anonymous-schema-334>"
                        },
                        "customer_id": {
                          "description": "Customer ID",
                          "type": "string",
                          "x-parser-schema-id": "<anonymous-schema-335>"
                        },
                        "order_id": {
                          "description": "Order ID",
                          "type": "string",
                          "x-parser-schema-id": "<anonymous-schema-336>"
                        },
                        "payment_id": {
                          "description": "Payment ID",
                          "type": "string",
                          "x-parser-schema-id": "<anonymous-schema-337>"
                        },
                        "payment_method": "$ref:$.components.messages.auth.login.payload.oneOf[27].properties.payment_method",
                        "server_id": {
                          "description": "Server ID",
                          "type": "string",
                          "x-parser-schema-id": "<anonymous-schema-338>"
                        },
                        "status": "$ref:$.components.messages.auth.login.payload.oneOf[29].properties.status.anyOf[0]",
                        "transaction_id": {
                          "description": "Payment gateway transaction ID",
                          "type": [
                            "string",
                            "null"
                          ],
                          "x-parser-schema-id": "<anonymous-schema-339>"
                        }
                      },
                      "required": [
                        "payment_id",
                        "order_id",
                        "server_id",
                        "customer_id",
                        "amount",
                        "payment_method",
                        "status",
                        "created_at"
                      ],
                      "type": "object",
                      "x-parser-schema-id": "PaymentData"
                    },
                    {
                      "type": "null",
                      "x-parser-schema-id": "<anonymous-schema-340>"
                    }
                  ],
                  "description": "Payment details",
                  "x-parser-schema-id": "<anonymous-schema-332>"
                },
                "payment_id": {
                  "description": "Payment ID",
                  "type": [
                    "string",
                    "null"
                  ],
                  "x-parser-schema-id": "<anonymous-schema-341>"
                },
                "payment_url": {
                  "description": "Payment gateway URL (for redirect)",
                  "type": [
                    "string",
                    "null"
                  ],
                  "x-parser-schema-id": "<anonymous-schema-342>"
                },
                "success": {
                  "description": "Whether creation was successful",
                  "type": "boolean",
                  "x-parser-schema-id": "<anonymous-schema-343>"
                }
              },
              "required": [
                "success"
              ],
              "type": "object",
              "x-parser-schema-id": "PaymentCreatedResponse"
            },
            {
              "description": "Payment verified response",
              "properties": {
                "error": {
                  "description": "Error if failed",
                  "type": [
                    "string",
                    "null"
                  ],
                  "x-parser-schema-id": "<anonymous-schema-344>"
                },
                "payment": {
                  "anyOf": [
                    "$ref:$.components.messages.auth.logout.response.payload.oneOf[27].properties.payment.anyOf[0]",
                    {
                      "type": "null",
                      "x-parser-schema-id": "<anonymous-schema-346>"
                    }
                  ],
                  "description": "Payment details",
                  "x-parser-schema-id": "<anonymous-schema-345>"
                },
                "payment_id": {
                  "description": "Payment ID",
                  "type": [
                    "string",
                    "null"
                  ],
                  "x-parser-schema-id": "<anonymous-schema-347>"
                },
                "success": {
                  "description": "Whether verification was successful",
                  "type": "boolean",
                  "x-parser-schema-id": "<anonymous-schema-348>"
                }
              },
              "required": [
                "success"
              ],
              "type": "object",
              "x-parser-schema-id": "PaymentVerifiedResponse"
            },
            {
              "description": "List payments response",
              "properties": {
                "has_more": {
                  "description": "Whether there are more payments",
                  "type": "boolean",
                  "x-parser-schema-id": "<anonymous-schema-349>"
                },
                "payments": {
                  "description": "List of payments",
                  "items": "$ref:$.components.messages.auth.logout.response.payload.oneOf[27].properties.payment.anyOf[0]",
                  "type": "array",
                  "x-parser-schema-id": "<anonymous-schema-350>"
                },
                "total": {
                  "description": "Total count",
                  "format": "uint",
                  "minimum": 0,
                  "type": "integer",
                  "x-parser-schema-id": "<anonymous-schema-351>"
                }
              },
              "required": [
                "payments",
                "total",
                "has_more"
              ],
              "type": "object",
              "x-parser-schema-id": "PaymentListResponse"
            },
            {
              "description": "Review created response",
              "properties": {
                "error": {
                  "description": "Error if failed",
                  "type": [
                    "string",
                    "null"
                  ],
                  "x-parser-schema-id": "<anonymous-schema-352>"
                },
                "review": {
                  "anyOf": [
                    {
                      "description": "Review data",
                      "properties": {
                        "content": {
                          "description": "Review content",
                          "type": "string",
                          "x-parser-schema-id": "<anonymous-schema-354>"
                        },
                        "created_at": {
                          "description": "Created timestamp",
                          "format": "date-time",
                          "type": "string",
                          "x-parser-schema-id": "<anonymous-schema-355>"
                        },
                        "helpful_count": {
                          "description": "Helpfulness count",
                          "format": "uint32",
                          "minimum": 0,
                          "type": "integer",
                          "x-parser-schema-id": "<anonymous-schema-356>"
                        },
                        "image_urls": {
                          "description": "Image URLs",
                          "items": {
                            "type": "string",
                            "x-parser-schema-id": "<anonymous-schema-358>"
                          },
                          "type": [
                            "array",
                            "null"
                          ],
                          "x-parser-schema-id": "<anonymous-schema-357>"
                        },
                        "is_verified_purchase": {
                          "description": "Whether review is verified purchase",
                          "type": "boolean",
                          "x-parser-schema-id": "<anonymous-schema-359>"
                        },
                        "order_id": {
                          "description": "Order ID",
                          "type": "string",
                          "x-parser-schema-id": "<anonymous-schema-360>"
                        },
                        "product_id": {
                          "description": "Product ID (optional)",
                          "type": [
                            "string",
                            "null"
                          ],
                          "x-parser-schema-id": "<anonymous-schema-361>"
                        },
                        "rating": {
                          "description": "Rating (1-5)",
                          "format": "uint8",
                          "maximum": 255,
                          "minimum": 0,
                          "type": "integer",
                          "x-parser-schema-id": "<anonymous-schema-362>"
                        },
                        "reply": {
                          "anyOf": [
                            {
                              "description": "Store owner reply to review",
                              "properties": {
                                "content": {
                                  "description": "Reply content",
                                  "type": "string",
                                  "x-parser-schema-id": "<anonymous-schema-364>"
                                },
                                "replied_at": {
                                  "description": "Reply timestamp",
                                  "format": "date-time",
                                  "type": "string",
                                  "x-parser-schema-id": "<anonymous-schema-365>"
                                },
                                "replied_by": {
                                  "description": "Who replied",
                                  "type": "string",
                                  "x-parser-schema-id": "<anonymous-schema-366>"
                                }
                              },
                              "required": [
                                "content",
                                "replied_by",
                                "replied_at"
                              ],
                              "type": "object",
                              "x-parser-schema-id": "ReviewReply"
                            },
                            {
                              "type": "null",
                              "x-parser-schema-id": "<anonymous-schema-367>"
                            }
                          ],
                          "description": "Store owner reply",
                          "x-parser-schema-id": "<anonymous-schema-363>"
                        },
                        "review_id": {
                          "description": "Review ID",
                          "type": "string",
                          "x-parser-schema-id": "<anonymous-schema-368>"
                        },
                        "reviewer_id": {
                          "description": "Reviewer member ID",
                          "type": "string",
                          "x-parser-schema-id": "<anonymous-schema-369>"
                        },
                        "reviewer_name": {
                          "description": "Reviewer display name",
                          "type": [
                            "string",
                            "null"
                          ],
                          "x-parser-schema-id": "<anonymous-schema-370>"
                        },
                        "server_id": {
                          "description": "Server ID",
                          "type": "string",
                          "x-parser-schema-id": "<anonymous-schema-371>"
                        },
                        "title": {
                          "description": "Review title",
                          "type": "string",
                          "x-parser-schema-id": "<anonymous-schema-372>"
                        },
                        "updated_at": {
                          "description": "Updated timestamp",
                          "format": "date-time",
                          "type": [
                            "string",
                            "null"
                          ],
                          "x-parser-schema-id": "<anonymous-schema-373>"
                        }
                      },
                      "required": [
                        "review_id",
                        "server_id",
                        "order_id",
                        "reviewer_id",
                        "rating",
                        "title",
                        "content",
                        "created_at",
                        "is_verified_purchase",
                        "helpful_count"
                      ],
                      "type": "object",
                      "x-parser-schema-id": "ReviewData"
                    },
                    {
                      "type": "null",
                      "x-parser-schema-id": "<anonymous-schema-374>"
                    }
                  ],
                  "description": "Review details",
                  "x-parser-schema-id": "<anonymous-schema-353>"
                },
                "review_id": {
                  "description": "Created review ID",
                  "type": [
                    "string",
                    "null"
                  ],
                  "x-parser-schema-id": "<anonymous-schema-375>"
                },
                "success": {
                  "description": "Whether creation was successful",
                  "type": "boolean",
                  "x-parser-schema-id": "<anonymous-schema-376>"
                }
              },
              "required": [
                "success"
              ],
              "type": "object",
              "x-parser-schema-id": "ReviewCreatedResponse"
            },
            {
              "description": "List reviews response",
              "properties": {
                "average_rating": {
                  "description": "Average rating",
                  "format": "float",
                  "type": [
                    "number",
                    "null"
                  ],
                  "x-parser-schema-id": "<anonymous-schema-377>"
                },
                "has_more": {
                  "description": "Whether there are more reviews",
                  "type": "boolean",
                  "x-parser-schema-id": "<anonymous-schema-378>"
                },
                "rating_distribution": {
                  "anyOf": [
                    {
                      "description": "Rating distribution",
                      "properties": {
                        "five_stars": {
                          "description": "Number of 5-star reviews",
                          "format": "uint32",
                          "minimum": 0,
                          "type": "integer",
                          "x-parser-schema-id": "<anonymous-schema-380>"
                        },
                        "four_stars": {
                          "description": "Number of 4-star reviews",
                          "format": "uint32",
                          "minimum": 0,
                          "type": "integer",
                          "x-parser-schema-id": "<anonymous-schema-381>"
                        },
                        "one_star": {
                          "description": "Number of 1-star reviews",
                          "format": "uint32",
                          "minimum": 0,
                          "type": "integer",
                          "x-parser-schema-id": "<anonymous-schema-382>"
                        },
                        "three_stars": {
                          "description": "Number of 3-star reviews",
                          "format": "uint32",
                          "minimum": 0,
                          "type": "integer",
                          "x-parser-schema-id": "<anonymous-schema-383>"
                        },
                        "two_stars": {
                          "description": "Number of 2-star reviews",
                          "format": "uint32",
                          "minimum": 0,
                          "type": "integer",
                          "x-parser-schema-id": "<anonymous-schema-384>"
                        }
                      },
                      "required": [
                        "five_stars",
                        "four_stars",
                        "three_stars",
                        "two_stars",
                        "one_star"
                      ],
                      "type": "object",
                      "x-parser-schema-id": "RatingDistribution"
                    },
                    {
                      "type": "null",
                      "x-parser-schema-id": "<anonymous-schema-385>"
                    }
                  ],
                  "description": "Rating distribution (1-5 stars)",
                  "x-parser-schema-id": "<anonymous-schema-379>"
                },
                "reviews": {
                  "description": "List of reviews",
                  "items": "$ref:$.components.messages.auth.logout.response.payload.oneOf[30].properties.review.anyOf[0]",
                  "type": "array",
                  "x-parser-schema-id": "<anonymous-schema-386>"
                },
                "total": {
                  "description": "Total count",
                  "format": "uint",
                  "minimum": 0,
                  "type": "integer",
                  "x-parser-schema-id": "<anonymous-schema-387>"
                }
              },
              "required": [
                "reviews",
                "total",
                "has_more"
              ],
              "type": "object",
              "x-parser-schema-id": "ReviewListResponse"
            },
            {
              "description": "Generic error response",
              "properties": {
                "code": {
                  "description": "Error code",
                  "type": "string",
                  "x-parser-schema-id": "<anonymous-schema-388>"
                },
                "details": {
                  "description": "Additional error details",
                  "x-parser-schema-id": "<anonymous-schema-389>"
                },
                "message": {
                  "description": "Human-readable error message",
                  "type": "string",
                  "x-parser-schema-id": "<anonymous-schema-390>"
                },
                "request_id": {
                  "description": "Optional request ID that caused the error",
                  "type": [
                    "string",
                    "null"
                  ],
                  "x-parser-schema-id": "<anonymous-schema-391>"
                }
              },
              "required": [
                "code",
                "message"
              ],
              "type": "object",
              "x-parser-schema-id": "ErrorResponse"
            },
            {
              "description": "Server ping message for keepalive",
              "properties": {
                "server_id": {
                  "description": "Server ID",
                  "type": "string",
                  "x-parser-schema-id": "<anonymous-schema-392>"
                },
                "timestamp": {
                  "description": "Server timestamp",
                  "format": "int64",
                  "type": "integer",
                  "x-parser-schema-id": "<anonymous-schema-393>"
                }
              },
              "required": [
                "timestamp",
                "server_id"
              ],
              "type": "object",
              "x-parser-schema-id": "ServerPingMessage"
            },
            {
              "description": "Connection status message",
              "properties": {
                "message": {
                  "description": "Message",
                  "type": [
                    "string",
                    "null"
                  ],
                  "x-parser-schema-id": "<anonymous-schema-394>"
                },
                "session_id": {
                  "description": "Session ID",
                  "type": "string",
                  "x-parser-schema-id": "<anonymous-schema-395>"
                },
                "status": {
                  "description": "Connection status enum",
                  "oneOf": [
                    {
                      "const": "connected",
                      "description": "Connected",
                      "type": "string",
                      "x-parser-schema-id": "<anonymous-schema-396>"
                    },
                    {
                      "const": "reconnecting",
                      "description": "Reconnecting",
                      "type": "string",
                      "x-parser-schema-id": "<anonymous-schema-397>"
                    },
                    {
                      "const": "disconnected",
                      "description": "Disconnected",
                      "type": "string",
                      "x-parser-schema-id": "<anonymous-schema-398>"
                    },
                    {
                      "const": "error",
                      "description": "Error",
                      "type": "string",
                      "x-parser-schema-id": "<anonymous-schema-399>"
                    }
                  ],
                  "x-parser-schema-id": "ConnectionStatus"
                }
              },
              "required": [
                "status",
                "session_id"
              ],
              "type": "object",
              "x-parser-schema-id": "ConnectionStatusMessage"
            }
          ],
          "title": "ServerMessage",
          "x-parser-schema-id": "<anonymous-schema-174>"
        },
        "title": "auth.logout.response",
        "x-parser-unique-object-id": "auth.logout.response"
      },
      "auth.response": {
        "contentType": "application/json",
        "name": "auth.response",
        "payload": {
          "description": "All server-to-client messages",
          "oneOf": [
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[0]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[1]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[2]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[3]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[4]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[5]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[6]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[7]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[8]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[9]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[10]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[11]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[12]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[13]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[14]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[15]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[16]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[17]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[18]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[19]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[20]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[21]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[22]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[23]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[24]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[25]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[26]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[27]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[28]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[29]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[30]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[31]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[32]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[33]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[34]"
          ],
          "title": "ServerMessage",
          "x-parser-schema-id": "<anonymous-schema-400>"
        },
        "title": "auth.response",
        "x-parser-unique-object-id": "auth.response"
      },
      "channel.create": {
        "contentType": "application/json",
        "name": "channel.create",
        "payload": {
          "description": "All client-to-server messages",
          "oneOf": [
            "$ref:$.components.messages.auth.login.payload.oneOf[0]",
            "$ref:$.components.messages.auth.login.payload.oneOf[1]",
            "$ref:$.components.messages.auth.login.payload.oneOf[2]",
            "$ref:$.components.messages.auth.login.payload.oneOf[3]",
            "$ref:$.components.messages.auth.login.payload.oneOf[4]",
            "$ref:$.components.messages.auth.login.payload.oneOf[5]",
            "$ref:$.components.messages.auth.login.payload.oneOf[6]",
            "$ref:$.components.messages.auth.login.payload.oneOf[7]",
            "$ref:$.components.messages.auth.login.payload.oneOf[8]",
            "$ref:$.components.messages.auth.login.payload.oneOf[9]",
            "$ref:$.components.messages.auth.login.payload.oneOf[10]",
            "$ref:$.components.messages.auth.login.payload.oneOf[11]",
            "$ref:$.components.messages.auth.login.payload.oneOf[12]",
            "$ref:$.components.messages.auth.login.payload.oneOf[13]",
            "$ref:$.components.messages.auth.login.payload.oneOf[14]",
            "$ref:$.components.messages.auth.login.payload.oneOf[15]",
            "$ref:$.components.messages.auth.login.payload.oneOf[16]",
            "$ref:$.components.messages.auth.login.payload.oneOf[17]",
            "$ref:$.components.messages.auth.login.payload.oneOf[18]",
            "$ref:$.components.messages.auth.login.payload.oneOf[19]",
            "$ref:$.components.messages.auth.login.payload.oneOf[20]",
            "$ref:$.components.messages.auth.login.payload.oneOf[21]",
            "$ref:$.components.messages.auth.login.payload.oneOf[22]",
            "$ref:$.components.messages.auth.login.payload.oneOf[23]",
            "$ref:$.components.messages.auth.login.payload.oneOf[24]",
            "$ref:$.components.messages.auth.login.payload.oneOf[25]",
            "$ref:$.components.messages.auth.login.payload.oneOf[26]",
            "$ref:$.components.messages.auth.login.payload.oneOf[27]",
            "$ref:$.components.messages.auth.login.payload.oneOf[28]",
            "$ref:$.components.messages.auth.login.payload.oneOf[29]",
            "$ref:$.components.messages.auth.login.payload.oneOf[30]",
            "$ref:$.components.messages.auth.login.payload.oneOf[31]"
          ],
          "title": "ClientMessage",
          "x-parser-schema-id": "<anonymous-schema-401>"
        },
        "title": "channel.create",
        "x-parser-unique-object-id": "channel.create"
      },
      "channel.created": {
        "contentType": "application/json",
        "name": "channel.created",
        "payload": {
          "description": "All server-to-client messages",
          "oneOf": [
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[0]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[1]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[2]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[3]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[4]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[5]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[6]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[7]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[8]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[9]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[10]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[11]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[12]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[13]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[14]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[15]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[16]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[17]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[18]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[19]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[20]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[21]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[22]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[23]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[24]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[25]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[26]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[27]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[28]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[29]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[30]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[31]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[32]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[33]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[34]"
          ],
          "title": "ServerMessage",
          "x-parser-schema-id": "<anonymous-schema-402>"
        },
        "title": "channel.created",
        "x-parser-unique-object-id": "channel.created"
      },
      "channel.delete": {
        "contentType": "application/json",
        "name": "channel.delete",
        "payload": {
          "description": "All client-to-server messages",
          "oneOf": [
            "$ref:$.components.messages.auth.login.payload.oneOf[0]",
            "$ref:$.components.messages.auth.login.payload.oneOf[1]",
            "$ref:$.components.messages.auth.login.payload.oneOf[2]",
            "$ref:$.components.messages.auth.login.payload.oneOf[3]",
            "$ref:$.components.messages.auth.login.payload.oneOf[4]",
            "$ref:$.components.messages.auth.login.payload.oneOf[5]",
            "$ref:$.components.messages.auth.login.payload.oneOf[6]",
            "$ref:$.components.messages.auth.login.payload.oneOf[7]",
            "$ref:$.components.messages.auth.login.payload.oneOf[8]",
            "$ref:$.components.messages.auth.login.payload.oneOf[9]",
            "$ref:$.components.messages.auth.login.payload.oneOf[10]",
            "$ref:$.components.messages.auth.login.payload.oneOf[11]",
            "$ref:$.components.messages.auth.login.payload.oneOf[12]",
            "$ref:$.components.messages.auth.login.payload.oneOf[13]",
            "$ref:$.components.messages.auth.login.payload.oneOf[14]",
            "$ref:$.components.messages.auth.login.payload.oneOf[15]",
            "$ref:$.components.messages.auth.login.payload.oneOf[16]",
            "$ref:$.components.messages.auth.login.payload.oneOf[17]",
            "$ref:$.components.messages.auth.login.payload.oneOf[18]",
            "$ref:$.components.messages.auth.login.payload.oneOf[19]",
            "$ref:$.components.messages.auth.login.payload.oneOf[20]",
            "$ref:$.components.messages.auth.login.payload.oneOf[21]",
            "$ref:$.components.messages.auth.login.payload.oneOf[22]",
            "$ref:$.components.messages.auth.login.payload.oneOf[23]",
            "$ref:$.components.messages.auth.login.payload.oneOf[24]",
            "$ref:$.components.messages.auth.login.payload.oneOf[25]",
            "$ref:$.components.messages.auth.login.payload.oneOf[26]",
            "$ref:$.components.messages.auth.login.payload.oneOf[27]",
            "$ref:$.components.messages.auth.login.payload.oneOf[28]",
            "$ref:$.components.messages.auth.login.payload.oneOf[29]",
            "$ref:$.components.messages.auth.login.payload.oneOf[30]",
            "$ref:$.components.messages.auth.login.payload.oneOf[31]"
          ],
          "title": "ClientMessage",
          "x-parser-schema-id": "<anonymous-schema-403>"
        },
        "title": "channel.delete",
        "x-parser-unique-object-id": "channel.delete"
      },
      "channel.deleted": {
        "contentType": "application/json",
        "name": "channel.deleted",
        "payload": {
          "description": "All server-to-client messages",
          "oneOf": [
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[0]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[1]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[2]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[3]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[4]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[5]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[6]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[7]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[8]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[9]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[10]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[11]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[12]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[13]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[14]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[15]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[16]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[17]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[18]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[19]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[20]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[21]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[22]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[23]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[24]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[25]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[26]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[27]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[28]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[29]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[30]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[31]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[32]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[33]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[34]"
          ],
          "title": "ServerMessage",
          "x-parser-schema-id": "<anonymous-schema-404>"
        },
        "title": "channel.deleted",
        "x-parser-unique-object-id": "channel.deleted"
      },
      "channel.edit": {
        "contentType": "application/json",
        "name": "channel.edit",
        "payload": {
          "description": "All client-to-server messages",
          "oneOf": [
            "$ref:$.components.messages.auth.login.payload.oneOf[0]",
            "$ref:$.components.messages.auth.login.payload.oneOf[1]",
            "$ref:$.components.messages.auth.login.payload.oneOf[2]",
            "$ref:$.components.messages.auth.login.payload.oneOf[3]",
            "$ref:$.components.messages.auth.login.payload.oneOf[4]",
            "$ref:$.components.messages.auth.login.payload.oneOf[5]",
            "$ref:$.components.messages.auth.login.payload.oneOf[6]",
            "$ref:$.components.messages.auth.login.payload.oneOf[7]",
            "$ref:$.components.messages.auth.login.payload.oneOf[8]",
            "$ref:$.components.messages.auth.login.payload.oneOf[9]",
            "$ref:$.components.messages.auth.login.payload.oneOf[10]",
            "$ref:$.components.messages.auth.login.payload.oneOf[11]",
            "$ref:$.components.messages.auth.login.payload.oneOf[12]",
            "$ref:$.components.messages.auth.login.payload.oneOf[13]",
            "$ref:$.components.messages.auth.login.payload.oneOf[14]",
            "$ref:$.components.messages.auth.login.payload.oneOf[15]",
            "$ref:$.components.messages.auth.login.payload.oneOf[16]",
            "$ref:$.components.messages.auth.login.payload.oneOf[17]",
            "$ref:$.components.messages.auth.login.payload.oneOf[18]",
            "$ref:$.components.messages.auth.login.payload.oneOf[19]",
            "$ref:$.components.messages.auth.login.payload.oneOf[20]",
            "$ref:$.components.messages.auth.login.payload.oneOf[21]",
            "$ref:$.components.messages.auth.login.payload.oneOf[22]",
            "$ref:$.components.messages.auth.login.payload.oneOf[23]",
            "$ref:$.components.messages.auth.login.payload.oneOf[24]",
            "$ref:$.components.messages.auth.login.payload.oneOf[25]",
            "$ref:$.components.messages.auth.login.payload.oneOf[26]",
            "$ref:$.components.messages.auth.login.payload.oneOf[27]",
            "$ref:$.components.messages.auth.login.payload.oneOf[28]",
            "$ref:$.components.messages.auth.login.payload.oneOf[29]",
            "$ref:$.components.messages.auth.login.payload.oneOf[30]",
            "$ref:$.components.messages.auth.login.payload.oneOf[31]"
          ],
          "title": "ClientMessage",
          "x-parser-schema-id": "<anonymous-schema-405>"
        },
        "title": "channel.edit",
        "x-parser-unique-object-id": "channel.edit"
      },
      "channel.edited": {
        "contentType": "application/json",
        "name": "channel.edited",
        "payload": {
          "description": "All server-to-client messages",
          "oneOf": [
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[0]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[1]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[2]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[3]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[4]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[5]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[6]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[7]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[8]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[9]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[10]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[11]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[12]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[13]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[14]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[15]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[16]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[17]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[18]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[19]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[20]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[21]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[22]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[23]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[24]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[25]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[26]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[27]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[28]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[29]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[30]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[31]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[32]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[33]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[34]"
          ],
          "title": "ServerMessage",
          "x-parser-schema-id": "<anonymous-schema-406>"
        },
        "title": "channel.edited",
        "x-parser-unique-object-id": "channel.edited"
      },
      "channel.info": {
        "contentType": "application/json",
        "name": "channel.info",
        "payload": {
          "description": "All client-to-server messages",
          "oneOf": [
            "$ref:$.components.messages.auth.login.payload.oneOf[0]",
            "$ref:$.components.messages.auth.login.payload.oneOf[1]",
            "$ref:$.components.messages.auth.login.payload.oneOf[2]",
            "$ref:$.components.messages.auth.login.payload.oneOf[3]",
            "$ref:$.components.messages.auth.login.payload.oneOf[4]",
            "$ref:$.components.messages.auth.login.payload.oneOf[5]",
            "$ref:$.components.messages.auth.login.payload.oneOf[6]",
            "$ref:$.components.messages.auth.login.payload.oneOf[7]",
            "$ref:$.components.messages.auth.login.payload.oneOf[8]",
            "$ref:$.components.messages.auth.login.payload.oneOf[9]",
            "$ref:$.components.messages.auth.login.payload.oneOf[10]",
            "$ref:$.components.messages.auth.login.payload.oneOf[11]",
            "$ref:$.components.messages.auth.login.payload.oneOf[12]",
            "$ref:$.components.messages.auth.login.payload.oneOf[13]",
            "$ref:$.components.messages.auth.login.payload.oneOf[14]",
            "$ref:$.components.messages.auth.login.payload.oneOf[15]",
            "$ref:$.components.messages.auth.login.payload.oneOf[16]",
            "$ref:$.components.messages.auth.login.payload.oneOf[17]",
            "$ref:$.components.messages.auth.login.payload.oneOf[18]",
            "$ref:$.components.messages.auth.login.payload.oneOf[19]",
            "$ref:$.components.messages.auth.login.payload.oneOf[20]",
            "$ref:$.components.messages.auth.login.payload.oneOf[21]",
            "$ref:$.components.messages.auth.login.payload.oneOf[22]",
            "$ref:$.components.messages.auth.login.payload.oneOf[23]",
            "$ref:$.components.messages.auth.login.payload.oneOf[24]",
            "$ref:$.components.messages.auth.login.payload.oneOf[25]",
            "$ref:$.components.messages.auth.login.payload.oneOf[26]",
            "$ref:$.components.messages.auth.login.payload.oneOf[27]",
            "$ref:$.components.messages.auth.login.payload.oneOf[28]",
            "$ref:$.components.messages.auth.login.payload.oneOf[29]",
            "$ref:$.components.messages.auth.login.payload.oneOf[30]",
            "$ref:$.components.messages.auth.login.payload.oneOf[31]"
          ],
          "title": "ClientMessage",
          "x-parser-schema-id": "<anonymous-schema-407>"
        },
        "title": "channel.info",
        "x-parser-unique-object-id": "channel.info"
      },
      "channel.info.response": {
        "contentType": "application/json",
        "name": "channel.info.response",
        "payload": {
          "description": "All server-to-client messages",
          "oneOf": [
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[0]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[1]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[2]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[3]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[4]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[5]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[6]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[7]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[8]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[9]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[10]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[11]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[12]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[13]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[14]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[15]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[16]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[17]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[18]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[19]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[20]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[21]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[22]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[23]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[24]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[25]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[26]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[27]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[28]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[29]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[30]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[31]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[32]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[33]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[34]"
          ],
          "title": "ServerMessage",
          "x-parser-schema-id": "<anonymous-schema-408>"
        },
        "title": "channel.info.response",
        "x-parser-unique-object-id": "channel.info.response"
      },
      "channel.list": {
        "contentType": "application/json",
        "name": "channel.list",
        "payload": {
          "description": "All client-to-server messages",
          "oneOf": [
            "$ref:$.components.messages.auth.login.payload.oneOf[0]",
            "$ref:$.components.messages.auth.login.payload.oneOf[1]",
            "$ref:$.components.messages.auth.login.payload.oneOf[2]",
            "$ref:$.components.messages.auth.login.payload.oneOf[3]",
            "$ref:$.components.messages.auth.login.payload.oneOf[4]",
            "$ref:$.components.messages.auth.login.payload.oneOf[5]",
            "$ref:$.components.messages.auth.login.payload.oneOf[6]",
            "$ref:$.components.messages.auth.login.payload.oneOf[7]",
            "$ref:$.components.messages.auth.login.payload.oneOf[8]",
            "$ref:$.components.messages.auth.login.payload.oneOf[9]",
            "$ref:$.components.messages.auth.login.payload.oneOf[10]",
            "$ref:$.components.messages.auth.login.payload.oneOf[11]",
            "$ref:$.components.messages.auth.login.payload.oneOf[12]",
            "$ref:$.components.messages.auth.login.payload.oneOf[13]",
            "$ref:$.components.messages.auth.login.payload.oneOf[14]",
            "$ref:$.components.messages.auth.login.payload.oneOf[15]",
            "$ref:$.components.messages.auth.login.payload.oneOf[16]",
            "$ref:$.components.messages.auth.login.payload.oneOf[17]",
            "$ref:$.components.messages.auth.login.payload.oneOf[18]",
            "$ref:$.components.messages.auth.login.payload.oneOf[19]",
            "$ref:$.components.messages.auth.login.payload.oneOf[20]",
            "$ref:$.components.messages.auth.login.payload.oneOf[21]",
            "$ref:$.components.messages.auth.login.payload.oneOf[22]",
            "$ref:$.components.messages.auth.login.payload.oneOf[23]",
            "$ref:$.components.messages.auth.login.payload.oneOf[24]",
            "$ref:$.components.messages.auth.login.payload.oneOf[25]",
            "$ref:$.components.messages.auth.login.payload.oneOf[26]",
            "$ref:$.components.messages.auth.login.payload.oneOf[27]",
            "$ref:$.components.messages.auth.login.payload.oneOf[28]",
            "$ref:$.components.messages.auth.login.payload.oneOf[29]",
            "$ref:$.components.messages.auth.login.payload.oneOf[30]",
            "$ref:$.components.messages.auth.login.payload.oneOf[31]"
          ],
          "title": "ClientMessage",
          "x-parser-schema-id": "<anonymous-schema-409>"
        },
        "title": "channel.list",
        "x-parser-unique-object-id": "channel.list"
      },
      "channel.list.response": {
        "contentType": "application/json",
        "name": "channel.list.response",
        "payload": {
          "description": "All server-to-client messages",
          "oneOf": [
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[0]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[1]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[2]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[3]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[4]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[5]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[6]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[7]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[8]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[9]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[10]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[11]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[12]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[13]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[14]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[15]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[16]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[17]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[18]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[19]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[20]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[21]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[22]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[23]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[24]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[25]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[26]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[27]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[28]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[29]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[30]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[31]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[32]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[33]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[34]"
          ],
          "title": "ServerMessage",
          "x-parser-schema-id": "<anonymous-schema-410>"
        },
        "title": "channel.list.response",
        "x-parser-unique-object-id": "channel.list.response"
      },
      "connection.status": {
        "contentType": "application/json",
        "name": "connection.status",
        "payload": {
          "description": "All server-to-client messages",
          "oneOf": [
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[0]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[1]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[2]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[3]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[4]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[5]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[6]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[7]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[8]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[9]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[10]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[11]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[12]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[13]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[14]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[15]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[16]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[17]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[18]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[19]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[20]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[21]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[22]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[23]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[24]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[25]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[26]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[27]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[28]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[29]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[30]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[31]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[32]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[33]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[34]"
          ],
          "title": "ServerMessage",
          "x-parser-schema-id": "<anonymous-schema-411>"
        },
        "title": "connection.status",
        "x-parser-unique-object-id": "connection.status"
      },
      "error": {
        "contentType": "application/json",
        "name": "error",
        "payload": {
          "description": "All server-to-client messages",
          "oneOf": [
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[0]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[1]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[2]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[3]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[4]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[5]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[6]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[7]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[8]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[9]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[10]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[11]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[12]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[13]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[14]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[15]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[16]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[17]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[18]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[19]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[20]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[21]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[22]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[23]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[24]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[25]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[26]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[27]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[28]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[29]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[30]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[31]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[32]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[33]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[34]"
          ],
          "title": "ServerMessage",
          "x-parser-schema-id": "<anonymous-schema-412>"
        },
        "title": "error",
        "x-parser-unique-object-id": "error"
      },
      "hosting.close": {
        "contentType": "application/json",
        "name": "hosting.close",
        "payload": {
          "description": "All client-to-server messages",
          "oneOf": [
            "$ref:$.components.messages.auth.login.payload.oneOf[0]",
            "$ref:$.components.messages.auth.login.payload.oneOf[1]",
            "$ref:$.components.messages.auth.login.payload.oneOf[2]",
            "$ref:$.components.messages.auth.login.payload.oneOf[3]",
            "$ref:$.components.messages.auth.login.payload.oneOf[4]",
            "$ref:$.components.messages.auth.login.payload.oneOf[5]",
            "$ref:$.components.messages.auth.login.payload.oneOf[6]",
            "$ref:$.components.messages.auth.login.payload.oneOf[7]",
            "$ref:$.components.messages.auth.login.payload.oneOf[8]",
            "$ref:$.components.messages.auth.login.payload.oneOf[9]",
            "$ref:$.components.messages.auth.login.payload.oneOf[10]",
            "$ref:$.components.messages.auth.login.payload.oneOf[11]",
            "$ref:$.components.messages.auth.login.payload.oneOf[12]",
            "$ref:$.components.messages.auth.login.payload.oneOf[13]",
            "$ref:$.components.messages.auth.login.payload.oneOf[14]",
            "$ref:$.components.messages.auth.login.payload.oneOf[15]",
            "$ref:$.components.messages.auth.login.payload.oneOf[16]",
            "$ref:$.components.messages.auth.login.payload.oneOf[17]",
            "$ref:$.components.messages.auth.login.payload.oneOf[18]",
            "$ref:$.components.messages.auth.login.payload.oneOf[19]",
            "$ref:$.components.messages.auth.login.payload.oneOf[20]",
            "$ref:$.components.messages.auth.login.payload.oneOf[21]",
            "$ref:$.components.messages.auth.login.payload.oneOf[22]",
            "$ref:$.components.messages.auth.login.payload.oneOf[23]",
            "$ref:$.components.messages.auth.login.payload.oneOf[24]",
            "$ref:$.components.messages.auth.login.payload.oneOf[25]",
            "$ref:$.components.messages.auth.login.payload.oneOf[26]",
            "$ref:$.components.messages.auth.login.payload.oneOf[27]",
            "$ref:$.components.messages.auth.login.payload.oneOf[28]",
            "$ref:$.components.messages.auth.login.payload.oneOf[29]",
            "$ref:$.components.messages.auth.login.payload.oneOf[30]",
            "$ref:$.components.messages.auth.login.payload.oneOf[31]"
          ],
          "title": "ClientMessage",
          "x-parser-schema-id": "<anonymous-schema-413>"
        },
        "title": "hosting.close",
        "x-parser-unique-object-id": "hosting.close"
      },
      "hosting.init": {
        "contentType": "application/json",
        "name": "hosting.init",
        "payload": {
          "description": "All client-to-server messages",
          "oneOf": [
            "$ref:$.components.messages.auth.login.payload.oneOf[0]",
            "$ref:$.components.messages.auth.login.payload.oneOf[1]",
            "$ref:$.components.messages.auth.login.payload.oneOf[2]",
            "$ref:$.components.messages.auth.login.payload.oneOf[3]",
            "$ref:$.components.messages.auth.login.payload.oneOf[4]",
            "$ref:$.components.messages.auth.login.payload.oneOf[5]",
            "$ref:$.components.messages.auth.login.payload.oneOf[6]",
            "$ref:$.components.messages.auth.login.payload.oneOf[7]",
            "$ref:$.components.messages.auth.login.payload.oneOf[8]",
            "$ref:$.components.messages.auth.login.payload.oneOf[9]",
            "$ref:$.components.messages.auth.login.payload.oneOf[10]",
            "$ref:$.components.messages.auth.login.payload.oneOf[11]",
            "$ref:$.components.messages.auth.login.payload.oneOf[12]",
            "$ref:$.components.messages.auth.login.payload.oneOf[13]",
            "$ref:$.components.messages.auth.login.payload.oneOf[14]",
            "$ref:$.components.messages.auth.login.payload.oneOf[15]",
            "$ref:$.components.messages.auth.login.payload.oneOf[16]",
            "$ref:$.components.messages.auth.login.payload.oneOf[17]",
            "$ref:$.components.messages.auth.login.payload.oneOf[18]",
            "$ref:$.components.messages.auth.login.payload.oneOf[19]",
            "$ref:$.components.messages.auth.login.payload.oneOf[20]",
            "$ref:$.components.messages.auth.login.payload.oneOf[21]",
            "$ref:$.components.messages.auth.login.payload.oneOf[22]",
            "$ref:$.components.messages.auth.login.payload.oneOf[23]",
            "$ref:$.components.messages.auth.login.payload.oneOf[24]",
            "$ref:$.components.messages.auth.login.payload.oneOf[25]",
            "$ref:$.components.messages.auth.login.payload.oneOf[26]",
            "$ref:$.components.messages.auth.login.payload.oneOf[27]",
            "$ref:$.components.messages.auth.login.payload.oneOf[28]",
            "$ref:$.components.messages.auth.login.payload.oneOf[29]",
            "$ref:$.components.messages.auth.login.payload.oneOf[30]",
            "$ref:$.components.messages.auth.login.payload.oneOf[31]"
          ],
          "title": "ClientMessage",
          "x-parser-schema-id": "<anonymous-schema-414>"
        },
        "title": "hosting.init",
        "x-parser-unique-object-id": "hosting.init"
      },
      "hosting.init.response": {
        "contentType": "application/json",
        "name": "hosting.init.response",
        "payload": {
          "description": "All server-to-client messages",
          "oneOf": [
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[0]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[1]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[2]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[3]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[4]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[5]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[6]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[7]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[8]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[9]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[10]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[11]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[12]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[13]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[14]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[15]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[16]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[17]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[18]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[19]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[20]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[21]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[22]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[23]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[24]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[25]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[26]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[27]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[28]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[29]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[30]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[31]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[32]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[33]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[34]"
          ],
          "title": "ServerMessage",
          "x-parser-schema-id": "<anonymous-schema-415>"
        },
        "title": "hosting.init.response",
        "x-parser-unique-object-id": "hosting.init.response"
      },
      "hosting.list": {
        "contentType": "application/json",
        "name": "hosting.list",
        "payload": {
          "description": "All client-to-server messages",
          "oneOf": [
            "$ref:$.components.messages.auth.login.payload.oneOf[0]",
            "$ref:$.components.messages.auth.login.payload.oneOf[1]",
            "$ref:$.components.messages.auth.login.payload.oneOf[2]",
            "$ref:$.components.messages.auth.login.payload.oneOf[3]",
            "$ref:$.components.messages.auth.login.payload.oneOf[4]",
            "$ref:$.components.messages.auth.login.payload.oneOf[5]",
            "$ref:$.components.messages.auth.login.payload.oneOf[6]",
            "$ref:$.components.messages.auth.login.payload.oneOf[7]",
            "$ref:$.components.messages.auth.login.payload.oneOf[8]",
            "$ref:$.components.messages.auth.login.payload.oneOf[9]",
            "$ref:$.components.messages.auth.login.payload.oneOf[10]",
            "$ref:$.components.messages.auth.login.payload.oneOf[11]",
            "$ref:$.components.messages.auth.login.payload.oneOf[12]",
            "$ref:$.components.messages.auth.login.payload.oneOf[13]",
            "$ref:$.components.messages.auth.login.payload.oneOf[14]",
            "$ref:$.components.messages.auth.login.payload.oneOf[15]",
            "$ref:$.components.messages.auth.login.payload.oneOf[16]",
            "$ref:$.components.messages.auth.login.payload.oneOf[17]",
            "$ref:$.components.messages.auth.login.payload.oneOf[18]",
            "$ref:$.components.messages.auth.login.payload.oneOf[19]",
            "$ref:$.components.messages.auth.login.payload.oneOf[20]",
            "$ref:$.components.messages.auth.login.payload.oneOf[21]",
            "$ref:$.components.messages.auth.login.payload.oneOf[22]",
            "$ref:$.components.messages.auth.login.payload.oneOf[23]",
            "$ref:$.components.messages.auth.login.payload.oneOf[24]",
            "$ref:$.components.messages.auth.login.payload.oneOf[25]",
            "$ref:$.components.messages.auth.login.payload.oneOf[26]",
            "$ref:$.components.messages.auth.login.payload.oneOf[27]",
            "$ref:$.components.messages.auth.login.payload.oneOf[28]",
            "$ref:$.components.messages.auth.login.payload.oneOf[29]",
            "$ref:$.components.messages.auth.login.payload.oneOf[30]",
            "$ref:$.components.messages.auth.login.payload.oneOf[31]"
          ],
          "title": "ClientMessage",
          "x-parser-schema-id": "<anonymous-schema-416>"
        },
        "title": "hosting.list",
        "x-parser-unique-object-id": "hosting.list"
      },
      "hosting.list.response": {
        "contentType": "application/json",
        "name": "hosting.list.response",
        "payload": {
          "description": "All server-to-client messages",
          "oneOf": [
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[0]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[1]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[2]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[3]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[4]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[5]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[6]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[7]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[8]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[9]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[10]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[11]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[12]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[13]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[14]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[15]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[16]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[17]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[18]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[19]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[20]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[21]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[22]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[23]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[24]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[25]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[26]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[27]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[28]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[29]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[30]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[31]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[32]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[33]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[34]"
          ],
          "title": "ServerMessage",
          "x-parser-schema-id": "<anonymous-schema-417>"
        },
        "title": "hosting.list.response",
        "x-parser-unique-object-id": "hosting.list.response"
      },
      "hosting.select": {
        "contentType": "application/json",
        "name": "hosting.select",
        "payload": {
          "description": "All client-to-server messages",
          "oneOf": [
            "$ref:$.components.messages.auth.login.payload.oneOf[0]",
            "$ref:$.components.messages.auth.login.payload.oneOf[1]",
            "$ref:$.components.messages.auth.login.payload.oneOf[2]",
            "$ref:$.components.messages.auth.login.payload.oneOf[3]",
            "$ref:$.components.messages.auth.login.payload.oneOf[4]",
            "$ref:$.components.messages.auth.login.payload.oneOf[5]",
            "$ref:$.components.messages.auth.login.payload.oneOf[6]",
            "$ref:$.components.messages.auth.login.payload.oneOf[7]",
            "$ref:$.components.messages.auth.login.payload.oneOf[8]",
            "$ref:$.components.messages.auth.login.payload.oneOf[9]",
            "$ref:$.components.messages.auth.login.payload.oneOf[10]",
            "$ref:$.components.messages.auth.login.payload.oneOf[11]",
            "$ref:$.components.messages.auth.login.payload.oneOf[12]",
            "$ref:$.components.messages.auth.login.payload.oneOf[13]",
            "$ref:$.components.messages.auth.login.payload.oneOf[14]",
            "$ref:$.components.messages.auth.login.payload.oneOf[15]",
            "$ref:$.components.messages.auth.login.payload.oneOf[16]",
            "$ref:$.components.messages.auth.login.payload.oneOf[17]",
            "$ref:$.components.messages.auth.login.payload.oneOf[18]",
            "$ref:$.components.messages.auth.login.payload.oneOf[19]",
            "$ref:$.components.messages.auth.login.payload.oneOf[20]",
            "$ref:$.components.messages.auth.login.payload.oneOf[21]",
            "$ref:$.components.messages.auth.login.payload.oneOf[22]",
            "$ref:$.components.messages.auth.login.payload.oneOf[23]",
            "$ref:$.components.messages.auth.login.payload.oneOf[24]",
            "$ref:$.components.messages.auth.login.payload.oneOf[25]",
            "$ref:$.components.messages.auth.login.payload.oneOf[26]",
            "$ref:$.components.messages.auth.login.payload.oneOf[27]",
            "$ref:$.components.messages.auth.login.payload.oneOf[28]",
            "$ref:$.components.messages.auth.login.payload.oneOf[29]",
            "$ref:$.components.messages.auth.login.payload.oneOf[30]",
            "$ref:$.components.messages.auth.login.payload.oneOf[31]"
          ],
          "title": "ClientMessage",
          "x-parser-schema-id": "<anonymous-schema-418>"
        },
        "title": "hosting.select",
        "x-parser-unique-object-id": "hosting.select"
      },
      "hosting.select.response": {
        "contentType": "application/json",
        "name": "hosting.select.response",
        "payload": {
          "description": "All server-to-client messages",
          "oneOf": [
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[0]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[1]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[2]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[3]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[4]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[5]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[6]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[7]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[8]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[9]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[10]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[11]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[12]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[13]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[14]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[15]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[16]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[17]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[18]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[19]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[20]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[21]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[22]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[23]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[24]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[25]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[26]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[27]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[28]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[29]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[30]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[31]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[32]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[33]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[34]"
          ],
          "title": "ServerMessage",
          "x-parser-schema-id": "<anonymous-schema-419>"
        },
        "title": "hosting.select.response",
        "x-parser-unique-object-id": "hosting.select.response"
      },
      "hosting.show": {
        "contentType": "application/json",
        "name": "hosting.show",
        "payload": {
          "description": "All client-to-server messages",
          "oneOf": [
            "$ref:$.components.messages.auth.login.payload.oneOf[0]",
            "$ref:$.components.messages.auth.login.payload.oneOf[1]",
            "$ref:$.components.messages.auth.login.payload.oneOf[2]",
            "$ref:$.components.messages.auth.login.payload.oneOf[3]",
            "$ref:$.components.messages.auth.login.payload.oneOf[4]",
            "$ref:$.components.messages.auth.login.payload.oneOf[5]",
            "$ref:$.components.messages.auth.login.payload.oneOf[6]",
            "$ref:$.components.messages.auth.login.payload.oneOf[7]",
            "$ref:$.components.messages.auth.login.payload.oneOf[8]",
            "$ref:$.components.messages.auth.login.payload.oneOf[9]",
            "$ref:$.components.messages.auth.login.payload.oneOf[10]",
            "$ref:$.components.messages.auth.login.payload.oneOf[11]",
            "$ref:$.components.messages.auth.login.payload.oneOf[12]",
            "$ref:$.components.messages.auth.login.payload.oneOf[13]",
            "$ref:$.components.messages.auth.login.payload.oneOf[14]",
            "$ref:$.components.messages.auth.login.payload.oneOf[15]",
            "$ref:$.components.messages.auth.login.payload.oneOf[16]",
            "$ref:$.components.messages.auth.login.payload.oneOf[17]",
            "$ref:$.components.messages.auth.login.payload.oneOf[18]",
            "$ref:$.components.messages.auth.login.payload.oneOf[19]",
            "$ref:$.components.messages.auth.login.payload.oneOf[20]",
            "$ref:$.components.messages.auth.login.payload.oneOf[21]",
            "$ref:$.components.messages.auth.login.payload.oneOf[22]",
            "$ref:$.components.messages.auth.login.payload.oneOf[23]",
            "$ref:$.components.messages.auth.login.payload.oneOf[24]",
            "$ref:$.components.messages.auth.login.payload.oneOf[25]",
            "$ref:$.components.messages.auth.login.payload.oneOf[26]",
            "$ref:$.components.messages.auth.login.payload.oneOf[27]",
            "$ref:$.components.messages.auth.login.payload.oneOf[28]",
            "$ref:$.components.messages.auth.login.payload.oneOf[29]",
            "$ref:$.components.messages.auth.login.payload.oneOf[30]",
            "$ref:$.components.messages.auth.login.payload.oneOf[31]"
          ],
          "title": "ClientMessage",
          "x-parser-schema-id": "<anonymous-schema-420>"
        },
        "title": "hosting.show",
        "x-parser-unique-object-id": "hosting.show"
      },
      "hosting.show.response": {
        "contentType": "application/json",
        "name": "hosting.show.response",
        "payload": {
          "description": "All server-to-client messages",
          "oneOf": [
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[0]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[1]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[2]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[3]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[4]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[5]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[6]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[7]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[8]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[9]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[10]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[11]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[12]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[13]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[14]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[15]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[16]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[17]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[18]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[19]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[20]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[21]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[22]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[23]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[24]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[25]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[26]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[27]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[28]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[29]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[30]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[31]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[32]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[33]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[34]"
          ],
          "title": "ServerMessage",
          "x-parser-schema-id": "<anonymous-schema-421>"
        },
        "title": "hosting.show.response",
        "x-parser-unique-object-id": "hosting.show.response"
      },
      "member.ban": {
        "contentType": "application/json",
        "name": "member.ban",
        "payload": {
          "description": "All client-to-server messages",
          "oneOf": [
            "$ref:$.components.messages.auth.login.payload.oneOf[0]",
            "$ref:$.components.messages.auth.login.payload.oneOf[1]",
            "$ref:$.components.messages.auth.login.payload.oneOf[2]",
            "$ref:$.components.messages.auth.login.payload.oneOf[3]",
            "$ref:$.components.messages.auth.login.payload.oneOf[4]",
            "$ref:$.components.messages.auth.login.payload.oneOf[5]",
            "$ref:$.components.messages.auth.login.payload.oneOf[6]",
            "$ref:$.components.messages.auth.login.payload.oneOf[7]",
            "$ref:$.components.messages.auth.login.payload.oneOf[8]",
            "$ref:$.components.messages.auth.login.payload.oneOf[9]",
            "$ref:$.components.messages.auth.login.payload.oneOf[10]",
            "$ref:$.components.messages.auth.login.payload.oneOf[11]",
            "$ref:$.components.messages.auth.login.payload.oneOf[12]",
            "$ref:$.components.messages.auth.login.payload.oneOf[13]",
            "$ref:$.components.messages.auth.login.payload.oneOf[14]",
            "$ref:$.components.messages.auth.login.payload.oneOf[15]",
            "$ref:$.components.messages.auth.login.payload.oneOf[16]",
            "$ref:$.components.messages.auth.login.payload.oneOf[17]",
            "$ref:$.components.messages.auth.login.payload.oneOf[18]",
            "$ref:$.components.messages.auth.login.payload.oneOf[19]",
            "$ref:$.components.messages.auth.login.payload.oneOf[20]",
            "$ref:$.components.messages.auth.login.payload.oneOf[21]",
            "$ref:$.components.messages.auth.login.payload.oneOf[22]",
            "$ref:$.components.messages.auth.login.payload.oneOf[23]",
            "$ref:$.components.messages.auth.login.payload.oneOf[24]",
            "$ref:$.components.messages.auth.login.payload.oneOf[25]",
            "$ref:$.components.messages.auth.login.payload.oneOf[26]",
            "$ref:$.components.messages.auth.login.payload.oneOf[27]",
            "$ref:$.components.messages.auth.login.payload.oneOf[28]",
            "$ref:$.components.messages.auth.login.payload.oneOf[29]",
            "$ref:$.components.messages.auth.login.payload.oneOf[30]",
            "$ref:$.components.messages.auth.login.payload.oneOf[31]"
          ],
          "title": "ClientMessage",
          "x-parser-schema-id": "<anonymous-schema-422>"
        },
        "title": "member.ban",
        "x-parser-unique-object-id": "member.ban"
      },
      "member.banned": {
        "contentType": "application/json",
        "name": "member.banned",
        "payload": {
          "description": "All server-to-client messages",
          "oneOf": [
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[0]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[1]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[2]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[3]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[4]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[5]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[6]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[7]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[8]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[9]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[10]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[11]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[12]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[13]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[14]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[15]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[16]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[17]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[18]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[19]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[20]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[21]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[22]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[23]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[24]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[25]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[26]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[27]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[28]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[29]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[30]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[31]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[32]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[33]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[34]"
          ],
          "title": "ServerMessage",
          "x-parser-schema-id": "<anonymous-schema-423>"
        },
        "title": "member.banned",
        "x-parser-unique-object-id": "member.banned"
      },
      "member.info": {
        "contentType": "application/json",
        "name": "member.info",
        "payload": {
          "description": "All client-to-server messages",
          "oneOf": [
            "$ref:$.components.messages.auth.login.payload.oneOf[0]",
            "$ref:$.components.messages.auth.login.payload.oneOf[1]",
            "$ref:$.components.messages.auth.login.payload.oneOf[2]",
            "$ref:$.components.messages.auth.login.payload.oneOf[3]",
            "$ref:$.components.messages.auth.login.payload.oneOf[4]",
            "$ref:$.components.messages.auth.login.payload.oneOf[5]",
            "$ref:$.components.messages.auth.login.payload.oneOf[6]",
            "$ref:$.components.messages.auth.login.payload.oneOf[7]",
            "$ref:$.components.messages.auth.login.payload.oneOf[8]",
            "$ref:$.components.messages.auth.login.payload.oneOf[9]",
            "$ref:$.components.messages.auth.login.payload.oneOf[10]",
            "$ref:$.components.messages.auth.login.payload.oneOf[11]",
            "$ref:$.components.messages.auth.login.payload.oneOf[12]",
            "$ref:$.components.messages.auth.login.payload.oneOf[13]",
            "$ref:$.components.messages.auth.login.payload.oneOf[14]",
            "$ref:$.components.messages.auth.login.payload.oneOf[15]",
            "$ref:$.components.messages.auth.login.payload.oneOf[16]",
            "$ref:$.components.messages.auth.login.payload.oneOf[17]",
            "$ref:$.components.messages.auth.login.payload.oneOf[18]",
            "$ref:$.components.messages.auth.login.payload.oneOf[19]",
            "$ref:$.components.messages.auth.login.payload.oneOf[20]",
            "$ref:$.components.messages.auth.login.payload.oneOf[21]",
            "$ref:$.components.messages.auth.login.payload.oneOf[22]",
            "$ref:$.components.messages.auth.login.payload.oneOf[23]",
            "$ref:$.components.messages.auth.login.payload.oneOf[24]",
            "$ref:$.components.messages.auth.login.payload.oneOf[25]",
            "$ref:$.components.messages.auth.login.payload.oneOf[26]",
            "$ref:$.components.messages.auth.login.payload.oneOf[27]",
            "$ref:$.components.messages.auth.login.payload.oneOf[28]",
            "$ref:$.components.messages.auth.login.payload.oneOf[29]",
            "$ref:$.components.messages.auth.login.payload.oneOf[30]",
            "$ref:$.components.messages.auth.login.payload.oneOf[31]"
          ],
          "title": "ClientMessage",
          "x-parser-schema-id": "<anonymous-schema-424>"
        },
        "title": "member.info",
        "x-parser-unique-object-id": "member.info"
      },
      "member.info.response": {
        "contentType": "application/json",
        "name": "member.info.response",
        "payload": {
          "description": "All server-to-client messages",
          "oneOf": [
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[0]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[1]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[2]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[3]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[4]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[5]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[6]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[7]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[8]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[9]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[10]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[11]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[12]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[13]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[14]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[15]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[16]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[17]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[18]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[19]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[20]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[21]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[22]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[23]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[24]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[25]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[26]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[27]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[28]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[29]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[30]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[31]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[32]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[33]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[34]"
          ],
          "title": "ServerMessage",
          "x-parser-schema-id": "<anonymous-schema-425>"
        },
        "title": "member.info.response",
        "x-parser-unique-object-id": "member.info.response"
      },
      "member.kick": {
        "contentType": "application/json",
        "name": "member.kick",
        "payload": {
          "description": "All client-to-server messages",
          "oneOf": [
            "$ref:$.components.messages.auth.login.payload.oneOf[0]",
            "$ref:$.components.messages.auth.login.payload.oneOf[1]",
            "$ref:$.components.messages.auth.login.payload.oneOf[2]",
            "$ref:$.components.messages.auth.login.payload.oneOf[3]",
            "$ref:$.components.messages.auth.login.payload.oneOf[4]",
            "$ref:$.components.messages.auth.login.payload.oneOf[5]",
            "$ref:$.components.messages.auth.login.payload.oneOf[6]",
            "$ref:$.components.messages.auth.login.payload.oneOf[7]",
            "$ref:$.components.messages.auth.login.payload.oneOf[8]",
            "$ref:$.components.messages.auth.login.payload.oneOf[9]",
            "$ref:$.components.messages.auth.login.payload.oneOf[10]",
            "$ref:$.components.messages.auth.login.payload.oneOf[11]",
            "$ref:$.components.messages.auth.login.payload.oneOf[12]",
            "$ref:$.components.messages.auth.login.payload.oneOf[13]",
            "$ref:$.components.messages.auth.login.payload.oneOf[14]",
            "$ref:$.components.messages.auth.login.payload.oneOf[15]",
            "$ref:$.components.messages.auth.login.payload.oneOf[16]",
            "$ref:$.components.messages.auth.login.payload.oneOf[17]",
            "$ref:$.components.messages.auth.login.payload.oneOf[18]",
            "$ref:$.components.messages.auth.login.payload.oneOf[19]",
            "$ref:$.components.messages.auth.login.payload.oneOf[20]",
            "$ref:$.components.messages.auth.login.payload.oneOf[21]",
            "$ref:$.components.messages.auth.login.payload.oneOf[22]",
            "$ref:$.components.messages.auth.login.payload.oneOf[23]",
            "$ref:$.components.messages.auth.login.payload.oneOf[24]",
            "$ref:$.components.messages.auth.login.payload.oneOf[25]",
            "$ref:$.components.messages.auth.login.payload.oneOf[26]",
            "$ref:$.components.messages.auth.login.payload.oneOf[27]",
            "$ref:$.components.messages.auth.login.payload.oneOf[28]",
            "$ref:$.components.messages.auth.login.payload.oneOf[29]",
            "$ref:$.components.messages.auth.login.payload.oneOf[30]",
            "$ref:$.components.messages.auth.login.payload.oneOf[31]"
          ],
          "title": "ClientMessage",
          "x-parser-schema-id": "<anonymous-schema-426>"
        },
        "title": "member.kick",
        "x-parser-unique-object-id": "member.kick"
      },
      "member.kicked": {
        "contentType": "application/json",
        "name": "member.kicked",
        "payload": {
          "description": "All server-to-client messages",
          "oneOf": [
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[0]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[1]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[2]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[3]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[4]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[5]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[6]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[7]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[8]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[9]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[10]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[11]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[12]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[13]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[14]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[15]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[16]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[17]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[18]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[19]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[20]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[21]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[22]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[23]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[24]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[25]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[26]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[27]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[28]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[29]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[30]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[31]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[32]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[33]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[34]"
          ],
          "title": "ServerMessage",
          "x-parser-schema-id": "<anonymous-schema-427>"
        },
        "title": "member.kicked",
        "x-parser-unique-object-id": "member.kicked"
      },
      "member.list": {
        "contentType": "application/json",
        "name": "member.list",
        "payload": {
          "description": "All client-to-server messages",
          "oneOf": [
            "$ref:$.components.messages.auth.login.payload.oneOf[0]",
            "$ref:$.components.messages.auth.login.payload.oneOf[1]",
            "$ref:$.components.messages.auth.login.payload.oneOf[2]",
            "$ref:$.components.messages.auth.login.payload.oneOf[3]",
            "$ref:$.components.messages.auth.login.payload.oneOf[4]",
            "$ref:$.components.messages.auth.login.payload.oneOf[5]",
            "$ref:$.components.messages.auth.login.payload.oneOf[6]",
            "$ref:$.components.messages.auth.login.payload.oneOf[7]",
            "$ref:$.components.messages.auth.login.payload.oneOf[8]",
            "$ref:$.components.messages.auth.login.payload.oneOf[9]",
            "$ref:$.components.messages.auth.login.payload.oneOf[10]",
            "$ref:$.components.messages.auth.login.payload.oneOf[11]",
            "$ref:$.components.messages.auth.login.payload.oneOf[12]",
            "$ref:$.components.messages.auth.login.payload.oneOf[13]",
            "$ref:$.components.messages.auth.login.payload.oneOf[14]",
            "$ref:$.components.messages.auth.login.payload.oneOf[15]",
            "$ref:$.components.messages.auth.login.payload.oneOf[16]",
            "$ref:$.components.messages.auth.login.payload.oneOf[17]",
            "$ref:$.components.messages.auth.login.payload.oneOf[18]",
            "$ref:$.components.messages.auth.login.payload.oneOf[19]",
            "$ref:$.components.messages.auth.login.payload.oneOf[20]",
            "$ref:$.components.messages.auth.login.payload.oneOf[21]",
            "$ref:$.components.messages.auth.login.payload.oneOf[22]",
            "$ref:$.components.messages.auth.login.payload.oneOf[23]",
            "$ref:$.components.messages.auth.login.payload.oneOf[24]",
            "$ref:$.components.messages.auth.login.payload.oneOf[25]",
            "$ref:$.components.messages.auth.login.payload.oneOf[26]",
            "$ref:$.components.messages.auth.login.payload.oneOf[27]",
            "$ref:$.components.messages.auth.login.payload.oneOf[28]",
            "$ref:$.components.messages.auth.login.payload.oneOf[29]",
            "$ref:$.components.messages.auth.login.payload.oneOf[30]",
            "$ref:$.components.messages.auth.login.payload.oneOf[31]"
          ],
          "title": "ClientMessage",
          "x-parser-schema-id": "<anonymous-schema-428>"
        },
        "title": "member.list",
        "x-parser-unique-object-id": "member.list"
      },
      "member.list.response": {
        "contentType": "application/json",
        "name": "member.list.response",
        "payload": {
          "description": "All server-to-client messages",
          "oneOf": [
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[0]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[1]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[2]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[3]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[4]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[5]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[6]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[7]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[8]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[9]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[10]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[11]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[12]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[13]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[14]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[15]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[16]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[17]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[18]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[19]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[20]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[21]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[22]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[23]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[24]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[25]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[26]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[27]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[28]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[29]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[30]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[31]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[32]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[33]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[34]"
          ],
          "title": "ServerMessage",
          "x-parser-schema-id": "<anonymous-schema-429>"
        },
        "title": "member.list.response",
        "x-parser-unique-object-id": "member.list.response"
      },
      "message.delete": {
        "contentType": "application/json",
        "name": "message.delete",
        "payload": {
          "description": "All client-to-server messages",
          "oneOf": [
            "$ref:$.components.messages.auth.login.payload.oneOf[0]",
            "$ref:$.components.messages.auth.login.payload.oneOf[1]",
            "$ref:$.components.messages.auth.login.payload.oneOf[2]",
            "$ref:$.components.messages.auth.login.payload.oneOf[3]",
            "$ref:$.components.messages.auth.login.payload.oneOf[4]",
            "$ref:$.components.messages.auth.login.payload.oneOf[5]",
            "$ref:$.components.messages.auth.login.payload.oneOf[6]",
            "$ref:$.components.messages.auth.login.payload.oneOf[7]",
            "$ref:$.components.messages.auth.login.payload.oneOf[8]",
            "$ref:$.components.messages.auth.login.payload.oneOf[9]",
            "$ref:$.components.messages.auth.login.payload.oneOf[10]",
            "$ref:$.components.messages.auth.login.payload.oneOf[11]",
            "$ref:$.components.messages.auth.login.payload.oneOf[12]",
            "$ref:$.components.messages.auth.login.payload.oneOf[13]",
            "$ref:$.components.messages.auth.login.payload.oneOf[14]",
            "$ref:$.components.messages.auth.login.payload.oneOf[15]",
            "$ref:$.components.messages.auth.login.payload.oneOf[16]",
            "$ref:$.components.messages.auth.login.payload.oneOf[17]",
            "$ref:$.components.messages.auth.login.payload.oneOf[18]",
            "$ref:$.components.messages.auth.login.payload.oneOf[19]",
            "$ref:$.components.messages.auth.login.payload.oneOf[20]",
            "$ref:$.components.messages.auth.login.payload.oneOf[21]",
            "$ref:$.components.messages.auth.login.payload.oneOf[22]",
            "$ref:$.components.messages.auth.login.payload.oneOf[23]",
            "$ref:$.components.messages.auth.login.payload.oneOf[24]",
            "$ref:$.components.messages.auth.login.payload.oneOf[25]",
            "$ref:$.components.messages.auth.login.payload.oneOf[26]",
            "$ref:$.components.messages.auth.login.payload.oneOf[27]",
            "$ref:$.components.messages.auth.login.payload.oneOf[28]",
            "$ref:$.components.messages.auth.login.payload.oneOf[29]",
            "$ref:$.components.messages.auth.login.payload.oneOf[30]",
            "$ref:$.components.messages.auth.login.payload.oneOf[31]"
          ],
          "title": "ClientMessage",
          "x-parser-schema-id": "<anonymous-schema-430>"
        },
        "title": "message.delete",
        "x-parser-unique-object-id": "message.delete"
      },
      "message.deleted": {
        "contentType": "application/json",
        "name": "message.deleted",
        "payload": {
          "description": "All server-to-client messages",
          "oneOf": [
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[0]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[1]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[2]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[3]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[4]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[5]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[6]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[7]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[8]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[9]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[10]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[11]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[12]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[13]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[14]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[15]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[16]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[17]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[18]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[19]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[20]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[21]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[22]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[23]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[24]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[25]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[26]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[27]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[28]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[29]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[30]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[31]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[32]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[33]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[34]"
          ],
          "title": "ServerMessage",
          "x-parser-schema-id": "<anonymous-schema-431>"
        },
        "title": "message.deleted",
        "x-parser-unique-object-id": "message.deleted"
      },
      "message.edit": {
        "contentType": "application/json",
        "name": "message.edit",
        "payload": {
          "description": "All client-to-server messages",
          "oneOf": [
            "$ref:$.components.messages.auth.login.payload.oneOf[0]",
            "$ref:$.components.messages.auth.login.payload.oneOf[1]",
            "$ref:$.components.messages.auth.login.payload.oneOf[2]",
            "$ref:$.components.messages.auth.login.payload.oneOf[3]",
            "$ref:$.components.messages.auth.login.payload.oneOf[4]",
            "$ref:$.components.messages.auth.login.payload.oneOf[5]",
            "$ref:$.components.messages.auth.login.payload.oneOf[6]",
            "$ref:$.components.messages.auth.login.payload.oneOf[7]",
            "$ref:$.components.messages.auth.login.payload.oneOf[8]",
            "$ref:$.components.messages.auth.login.payload.oneOf[9]",
            "$ref:$.components.messages.auth.login.payload.oneOf[10]",
            "$ref:$.components.messages.auth.login.payload.oneOf[11]",
            "$ref:$.components.messages.auth.login.payload.oneOf[12]",
            "$ref:$.components.messages.auth.login.payload.oneOf[13]",
            "$ref:$.components.messages.auth.login.payload.oneOf[14]",
            "$ref:$.components.messages.auth.login.payload.oneOf[15]",
            "$ref:$.components.messages.auth.login.payload.oneOf[16]",
            "$ref:$.components.messages.auth.login.payload.oneOf[17]",
            "$ref:$.components.messages.auth.login.payload.oneOf[18]",
            "$ref:$.components.messages.auth.login.payload.oneOf[19]",
            "$ref:$.components.messages.auth.login.payload.oneOf[20]",
            "$ref:$.components.messages.auth.login.payload.oneOf[21]",
            "$ref:$.components.messages.auth.login.payload.oneOf[22]",
            "$ref:$.components.messages.auth.login.payload.oneOf[23]",
            "$ref:$.components.messages.auth.login.payload.oneOf[24]",
            "$ref:$.components.messages.auth.login.payload.oneOf[25]",
            "$ref:$.components.messages.auth.login.payload.oneOf[26]",
            "$ref:$.components.messages.auth.login.payload.oneOf[27]",
            "$ref:$.components.messages.auth.login.payload.oneOf[28]",
            "$ref:$.components.messages.auth.login.payload.oneOf[29]",
            "$ref:$.components.messages.auth.login.payload.oneOf[30]",
            "$ref:$.components.messages.auth.login.payload.oneOf[31]"
          ],
          "title": "ClientMessage",
          "x-parser-schema-id": "<anonymous-schema-432>"
        },
        "title": "message.edit",
        "x-parser-unique-object-id": "message.edit"
      },
      "message.edited": {
        "contentType": "application/json",
        "name": "message.edited",
        "payload": {
          "description": "All server-to-client messages",
          "oneOf": [
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[0]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[1]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[2]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[3]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[4]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[5]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[6]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[7]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[8]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[9]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[10]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[11]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[12]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[13]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[14]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[15]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[16]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[17]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[18]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[19]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[20]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[21]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[22]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[23]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[24]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[25]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[26]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[27]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[28]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[29]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[30]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[31]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[32]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[33]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[34]"
          ],
          "title": "ServerMessage",
          "x-parser-schema-id": "<anonymous-schema-433>"
        },
        "title": "message.edited",
        "x-parser-unique-object-id": "message.edited"
      },
      "message.list": {
        "contentType": "application/json",
        "name": "message.list",
        "payload": {
          "description": "All client-to-server messages",
          "oneOf": [
            "$ref:$.components.messages.auth.login.payload.oneOf[0]",
            "$ref:$.components.messages.auth.login.payload.oneOf[1]",
            "$ref:$.components.messages.auth.login.payload.oneOf[2]",
            "$ref:$.components.messages.auth.login.payload.oneOf[3]",
            "$ref:$.components.messages.auth.login.payload.oneOf[4]",
            "$ref:$.components.messages.auth.login.payload.oneOf[5]",
            "$ref:$.components.messages.auth.login.payload.oneOf[6]",
            "$ref:$.components.messages.auth.login.payload.oneOf[7]",
            "$ref:$.components.messages.auth.login.payload.oneOf[8]",
            "$ref:$.components.messages.auth.login.payload.oneOf[9]",
            "$ref:$.components.messages.auth.login.payload.oneOf[10]",
            "$ref:$.components.messages.auth.login.payload.oneOf[11]",
            "$ref:$.components.messages.auth.login.payload.oneOf[12]",
            "$ref:$.components.messages.auth.login.payload.oneOf[13]",
            "$ref:$.components.messages.auth.login.payload.oneOf[14]",
            "$ref:$.components.messages.auth.login.payload.oneOf[15]",
            "$ref:$.components.messages.auth.login.payload.oneOf[16]",
            "$ref:$.components.messages.auth.login.payload.oneOf[17]",
            "$ref:$.components.messages.auth.login.payload.oneOf[18]",
            "$ref:$.components.messages.auth.login.payload.oneOf[19]",
            "$ref:$.components.messages.auth.login.payload.oneOf[20]",
            "$ref:$.components.messages.auth.login.payload.oneOf[21]",
            "$ref:$.components.messages.auth.login.payload.oneOf[22]",
            "$ref:$.components.messages.auth.login.payload.oneOf[23]",
            "$ref:$.components.messages.auth.login.payload.oneOf[24]",
            "$ref:$.components.messages.auth.login.payload.oneOf[25]",
            "$ref:$.components.messages.auth.login.payload.oneOf[26]",
            "$ref:$.components.messages.auth.login.payload.oneOf[27]",
            "$ref:$.components.messages.auth.login.payload.oneOf[28]",
            "$ref:$.components.messages.auth.login.payload.oneOf[29]",
            "$ref:$.components.messages.auth.login.payload.oneOf[30]",
            "$ref:$.components.messages.auth.login.payload.oneOf[31]"
          ],
          "title": "ClientMessage",
          "x-parser-schema-id": "<anonymous-schema-434>"
        },
        "title": "message.list",
        "x-parser-unique-object-id": "message.list"
      },
      "message.list.response": {
        "contentType": "application/json",
        "name": "message.list.response",
        "payload": {
          "description": "All server-to-client messages",
          "oneOf": [
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[0]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[1]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[2]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[3]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[4]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[5]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[6]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[7]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[8]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[9]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[10]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[11]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[12]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[13]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[14]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[15]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[16]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[17]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[18]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[19]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[20]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[21]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[22]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[23]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[24]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[25]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[26]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[27]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[28]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[29]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[30]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[31]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[32]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[33]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[34]"
          ],
          "title": "ServerMessage",
          "x-parser-schema-id": "<anonymous-schema-435>"
        },
        "title": "message.list.response",
        "x-parser-unique-object-id": "message.list.response"
      },
      "message.received": {
        "contentType": "application/json",
        "name": "message.received",
        "payload": {
          "description": "All server-to-client messages",
          "oneOf": [
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[0]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[1]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[2]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[3]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[4]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[5]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[6]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[7]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[8]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[9]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[10]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[11]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[12]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[13]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[14]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[15]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[16]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[17]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[18]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[19]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[20]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[21]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[22]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[23]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[24]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[25]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[26]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[27]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[28]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[29]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[30]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[31]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[32]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[33]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[34]"
          ],
          "title": "ServerMessage",
          "x-parser-schema-id": "<anonymous-schema-436>"
        },
        "title": "message.received",
        "x-parser-unique-object-id": "message.received"
      },
      "message.reply": {
        "contentType": "application/json",
        "name": "message.reply",
        "payload": {
          "description": "All client-to-server messages",
          "oneOf": [
            "$ref:$.components.messages.auth.login.payload.oneOf[0]",
            "$ref:$.components.messages.auth.login.payload.oneOf[1]",
            "$ref:$.components.messages.auth.login.payload.oneOf[2]",
            "$ref:$.components.messages.auth.login.payload.oneOf[3]",
            "$ref:$.components.messages.auth.login.payload.oneOf[4]",
            "$ref:$.components.messages.auth.login.payload.oneOf[5]",
            "$ref:$.components.messages.auth.login.payload.oneOf[6]",
            "$ref:$.components.messages.auth.login.payload.oneOf[7]",
            "$ref:$.components.messages.auth.login.payload.oneOf[8]",
            "$ref:$.components.messages.auth.login.payload.oneOf[9]",
            "$ref:$.components.messages.auth.login.payload.oneOf[10]",
            "$ref:$.components.messages.auth.login.payload.oneOf[11]",
            "$ref:$.components.messages.auth.login.payload.oneOf[12]",
            "$ref:$.components.messages.auth.login.payload.oneOf[13]",
            "$ref:$.components.messages.auth.login.payload.oneOf[14]",
            "$ref:$.components.messages.auth.login.payload.oneOf[15]",
            "$ref:$.components.messages.auth.login.payload.oneOf[16]",
            "$ref:$.components.messages.auth.login.payload.oneOf[17]",
            "$ref:$.components.messages.auth.login.payload.oneOf[18]",
            "$ref:$.components.messages.auth.login.payload.oneOf[19]",
            "$ref:$.components.messages.auth.login.payload.oneOf[20]",
            "$ref:$.components.messages.auth.login.payload.oneOf[21]",
            "$ref:$.components.messages.auth.login.payload.oneOf[22]",
            "$ref:$.components.messages.auth.login.payload.oneOf[23]",
            "$ref:$.components.messages.auth.login.payload.oneOf[24]",
            "$ref:$.components.messages.auth.login.payload.oneOf[25]",
            "$ref:$.components.messages.auth.login.payload.oneOf[26]",
            "$ref:$.components.messages.auth.login.payload.oneOf[27]",
            "$ref:$.components.messages.auth.login.payload.oneOf[28]",
            "$ref:$.components.messages.auth.login.payload.oneOf[29]",
            "$ref:$.components.messages.auth.login.payload.oneOf[30]",
            "$ref:$.components.messages.auth.login.payload.oneOf[31]"
          ],
          "title": "ClientMessage",
          "x-parser-schema-id": "<anonymous-schema-437>"
        },
        "title": "message.reply",
        "x-parser-unique-object-id": "message.reply"
      },
      "message.send": {
        "contentType": "application/json",
        "name": "message.send",
        "payload": {
          "description": "All client-to-server messages",
          "oneOf": [
            "$ref:$.components.messages.auth.login.payload.oneOf[0]",
            "$ref:$.components.messages.auth.login.payload.oneOf[1]",
            "$ref:$.components.messages.auth.login.payload.oneOf[2]",
            "$ref:$.components.messages.auth.login.payload.oneOf[3]",
            "$ref:$.components.messages.auth.login.payload.oneOf[4]",
            "$ref:$.components.messages.auth.login.payload.oneOf[5]",
            "$ref:$.components.messages.auth.login.payload.oneOf[6]",
            "$ref:$.components.messages.auth.login.payload.oneOf[7]",
            "$ref:$.components.messages.auth.login.payload.oneOf[8]",
            "$ref:$.components.messages.auth.login.payload.oneOf[9]",
            "$ref:$.components.messages.auth.login.payload.oneOf[10]",
            "$ref:$.components.messages.auth.login.payload.oneOf[11]",
            "$ref:$.components.messages.auth.login.payload.oneOf[12]",
            "$ref:$.components.messages.auth.login.payload.oneOf[13]",
            "$ref:$.components.messages.auth.login.payload.oneOf[14]",
            "$ref:$.components.messages.auth.login.payload.oneOf[15]",
            "$ref:$.components.messages.auth.login.payload.oneOf[16]",
            "$ref:$.components.messages.auth.login.payload.oneOf[17]",
            "$ref:$.components.messages.auth.login.payload.oneOf[18]",
            "$ref:$.components.messages.auth.login.payload.oneOf[19]",
            "$ref:$.components.messages.auth.login.payload.oneOf[20]",
            "$ref:$.components.messages.auth.login.payload.oneOf[21]",
            "$ref:$.components.messages.auth.login.payload.oneOf[22]",
            "$ref:$.components.messages.auth.login.payload.oneOf[23]",
            "$ref:$.components.messages.auth.login.payload.oneOf[24]",
            "$ref:$.components.messages.auth.login.payload.oneOf[25]",
            "$ref:$.components.messages.auth.login.payload.oneOf[26]",
            "$ref:$.components.messages.auth.login.payload.oneOf[27]",
            "$ref:$.components.messages.auth.login.payload.oneOf[28]",
            "$ref:$.components.messages.auth.login.payload.oneOf[29]",
            "$ref:$.components.messages.auth.login.payload.oneOf[30]",
            "$ref:$.components.messages.auth.login.payload.oneOf[31]"
          ],
          "title": "ClientMessage",
          "x-parser-schema-id": "<anonymous-schema-438>"
        },
        "title": "message.send",
        "x-parser-unique-object-id": "message.send"
      },
      "message.sent": {
        "contentType": "application/json",
        "name": "message.sent",
        "payload": {
          "description": "All server-to-client messages",
          "oneOf": [
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[0]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[1]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[2]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[3]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[4]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[5]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[6]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[7]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[8]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[9]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[10]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[11]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[12]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[13]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[14]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[15]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[16]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[17]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[18]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[19]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[20]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[21]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[22]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[23]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[24]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[25]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[26]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[27]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[28]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[29]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[30]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[31]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[32]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[33]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[34]"
          ],
          "title": "ServerMessage",
          "x-parser-schema-id": "<anonymous-schema-439>"
        },
        "title": "message.sent",
        "x-parser-unique-object-id": "message.sent"
      },
      "order.create": {
        "contentType": "application/json",
        "name": "order.create",
        "payload": {
          "description": "All client-to-server messages",
          "oneOf": [
            "$ref:$.components.messages.auth.login.payload.oneOf[0]",
            "$ref:$.components.messages.auth.login.payload.oneOf[1]",
            "$ref:$.components.messages.auth.login.payload.oneOf[2]",
            "$ref:$.components.messages.auth.login.payload.oneOf[3]",
            "$ref:$.components.messages.auth.login.payload.oneOf[4]",
            "$ref:$.components.messages.auth.login.payload.oneOf[5]",
            "$ref:$.components.messages.auth.login.payload.oneOf[6]",
            "$ref:$.components.messages.auth.login.payload.oneOf[7]",
            "$ref:$.components.messages.auth.login.payload.oneOf[8]",
            "$ref:$.components.messages.auth.login.payload.oneOf[9]",
            "$ref:$.components.messages.auth.login.payload.oneOf[10]",
            "$ref:$.components.messages.auth.login.payload.oneOf[11]",
            "$ref:$.components.messages.auth.login.payload.oneOf[12]",
            "$ref:$.components.messages.auth.login.payload.oneOf[13]",
            "$ref:$.components.messages.auth.login.payload.oneOf[14]",
            "$ref:$.components.messages.auth.login.payload.oneOf[15]",
            "$ref:$.components.messages.auth.login.payload.oneOf[16]",
            "$ref:$.components.messages.auth.login.payload.oneOf[17]",
            "$ref:$.components.messages.auth.login.payload.oneOf[18]",
            "$ref:$.components.messages.auth.login.payload.oneOf[19]",
            "$ref:$.components.messages.auth.login.payload.oneOf[20]",
            "$ref:$.components.messages.auth.login.payload.oneOf[21]",
            "$ref:$.components.messages.auth.login.payload.oneOf[22]",
            "$ref:$.components.messages.auth.login.payload.oneOf[23]",
            "$ref:$.components.messages.auth.login.payload.oneOf[24]",
            "$ref:$.components.messages.auth.login.payload.oneOf[25]",
            "$ref:$.components.messages.auth.login.payload.oneOf[26]",
            "$ref:$.components.messages.auth.login.payload.oneOf[27]",
            "$ref:$.components.messages.auth.login.payload.oneOf[28]",
            "$ref:$.components.messages.auth.login.payload.oneOf[29]",
            "$ref:$.components.messages.auth.login.payload.oneOf[30]",
            "$ref:$.components.messages.auth.login.payload.oneOf[31]"
          ],
          "title": "ClientMessage",
          "x-parser-schema-id": "<anonymous-schema-440>"
        },
        "title": "order.create",
        "x-parser-unique-object-id": "order.create"
      },
      "order.created": {
        "contentType": "application/json",
        "name": "order.created",
        "payload": {
          "description": "All server-to-client messages",
          "oneOf": [
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[0]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[1]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[2]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[3]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[4]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[5]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[6]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[7]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[8]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[9]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[10]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[11]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[12]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[13]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[14]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[15]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[16]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[17]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[18]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[19]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[20]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[21]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[22]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[23]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[24]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[25]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[26]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[27]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[28]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[29]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[30]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[31]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[32]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[33]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[34]"
          ],
          "title": "ServerMessage",
          "x-parser-schema-id": "<anonymous-schema-441>"
        },
        "title": "order.created",
        "x-parser-unique-object-id": "order.created"
      },
      "order.list": {
        "contentType": "application/json",
        "name": "order.list",
        "payload": {
          "description": "All client-to-server messages",
          "oneOf": [
            "$ref:$.components.messages.auth.login.payload.oneOf[0]",
            "$ref:$.components.messages.auth.login.payload.oneOf[1]",
            "$ref:$.components.messages.auth.login.payload.oneOf[2]",
            "$ref:$.components.messages.auth.login.payload.oneOf[3]",
            "$ref:$.components.messages.auth.login.payload.oneOf[4]",
            "$ref:$.components.messages.auth.login.payload.oneOf[5]",
            "$ref:$.components.messages.auth.login.payload.oneOf[6]",
            "$ref:$.components.messages.auth.login.payload.oneOf[7]",
            "$ref:$.components.messages.auth.login.payload.oneOf[8]",
            "$ref:$.components.messages.auth.login.payload.oneOf[9]",
            "$ref:$.components.messages.auth.login.payload.oneOf[10]",
            "$ref:$.components.messages.auth.login.payload.oneOf[11]",
            "$ref:$.components.messages.auth.login.payload.oneOf[12]",
            "$ref:$.components.messages.auth.login.payload.oneOf[13]",
            "$ref:$.components.messages.auth.login.payload.oneOf[14]",
            "$ref:$.components.messages.auth.login.payload.oneOf[15]",
            "$ref:$.components.messages.auth.login.payload.oneOf[16]",
            "$ref:$.components.messages.auth.login.payload.oneOf[17]",
            "$ref:$.components.messages.auth.login.payload.oneOf[18]",
            "$ref:$.components.messages.auth.login.payload.oneOf[19]",
            "$ref:$.components.messages.auth.login.payload.oneOf[20]",
            "$ref:$.components.messages.auth.login.payload.oneOf[21]",
            "$ref:$.components.messages.auth.login.payload.oneOf[22]",
            "$ref:$.components.messages.auth.login.payload.oneOf[23]",
            "$ref:$.components.messages.auth.login.payload.oneOf[24]",
            "$ref:$.components.messages.auth.login.payload.oneOf[25]",
            "$ref:$.components.messages.auth.login.payload.oneOf[26]",
            "$ref:$.components.messages.auth.login.payload.oneOf[27]",
            "$ref:$.components.messages.auth.login.payload.oneOf[28]",
            "$ref:$.components.messages.auth.login.payload.oneOf[29]",
            "$ref:$.components.messages.auth.login.payload.oneOf[30]",
            "$ref:$.components.messages.auth.login.payload.oneOf[31]"
          ],
          "title": "ClientMessage",
          "x-parser-schema-id": "<anonymous-schema-442>"
        },
        "title": "order.list",
        "x-parser-unique-object-id": "order.list"
      },
      "order.list.response": {
        "contentType": "application/json",
        "name": "order.list.response",
        "payload": {
          "description": "All server-to-client messages",
          "oneOf": [
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[0]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[1]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[2]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[3]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[4]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[5]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[6]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[7]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[8]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[9]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[10]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[11]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[12]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[13]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[14]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[15]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[16]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[17]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[18]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[19]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[20]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[21]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[22]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[23]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[24]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[25]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[26]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[27]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[28]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[29]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[30]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[31]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[32]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[33]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[34]"
          ],
          "title": "ServerMessage",
          "x-parser-schema-id": "<anonymous-schema-443>"
        },
        "title": "order.list.response",
        "x-parser-unique-object-id": "order.list.response"
      },
      "order.status.update": {
        "contentType": "application/json",
        "name": "order.status.update",
        "payload": {
          "description": "All server-to-client messages",
          "oneOf": [
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[0]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[1]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[2]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[3]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[4]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[5]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[6]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[7]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[8]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[9]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[10]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[11]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[12]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[13]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[14]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[15]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[16]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[17]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[18]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[19]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[20]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[21]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[22]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[23]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[24]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[25]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[26]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[27]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[28]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[29]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[30]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[31]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[32]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[33]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[34]"
          ],
          "title": "ServerMessage",
          "x-parser-schema-id": "<anonymous-schema-444>"
        },
        "title": "order.status.update",
        "x-parser-unique-object-id": "order.status.update"
      },
      "payment.create": {
        "contentType": "application/json",
        "name": "payment.create",
        "payload": {
          "description": "All client-to-server messages",
          "oneOf": [
            "$ref:$.components.messages.auth.login.payload.oneOf[0]",
            "$ref:$.components.messages.auth.login.payload.oneOf[1]",
            "$ref:$.components.messages.auth.login.payload.oneOf[2]",
            "$ref:$.components.messages.auth.login.payload.oneOf[3]",
            "$ref:$.components.messages.auth.login.payload.oneOf[4]",
            "$ref:$.components.messages.auth.login.payload.oneOf[5]",
            "$ref:$.components.messages.auth.login.payload.oneOf[6]",
            "$ref:$.components.messages.auth.login.payload.oneOf[7]",
            "$ref:$.components.messages.auth.login.payload.oneOf[8]",
            "$ref:$.components.messages.auth.login.payload.oneOf[9]",
            "$ref:$.components.messages.auth.login.payload.oneOf[10]",
            "$ref:$.components.messages.auth.login.payload.oneOf[11]",
            "$ref:$.components.messages.auth.login.payload.oneOf[12]",
            "$ref:$.components.messages.auth.login.payload.oneOf[13]",
            "$ref:$.components.messages.auth.login.payload.oneOf[14]",
            "$ref:$.components.messages.auth.login.payload.oneOf[15]",
            "$ref:$.components.messages.auth.login.payload.oneOf[16]",
            "$ref:$.components.messages.auth.login.payload.oneOf[17]",
            "$ref:$.components.messages.auth.login.payload.oneOf[18]",
            "$ref:$.components.messages.auth.login.payload.oneOf[19]",
            "$ref:$.components.messages.auth.login.payload.oneOf[20]",
            "$ref:$.components.messages.auth.login.payload.oneOf[21]",
            "$ref:$.components.messages.auth.login.payload.oneOf[22]",
            "$ref:$.components.messages.auth.login.payload.oneOf[23]",
            "$ref:$.components.messages.auth.login.payload.oneOf[24]",
            "$ref:$.components.messages.auth.login.payload.oneOf[25]",
            "$ref:$.components.messages.auth.login.payload.oneOf[26]",
            "$ref:$.components.messages.auth.login.payload.oneOf[27]",
            "$ref:$.components.messages.auth.login.payload.oneOf[28]",
            "$ref:$.components.messages.auth.login.payload.oneOf[29]",
            "$ref:$.components.messages.auth.login.payload.oneOf[30]",
            "$ref:$.components.messages.auth.login.payload.oneOf[31]"
          ],
          "title": "ClientMessage",
          "x-parser-schema-id": "<anonymous-schema-445>"
        },
        "title": "payment.create",
        "x-parser-unique-object-id": "payment.create"
      },
      "payment.created": {
        "contentType": "application/json",
        "name": "payment.created",
        "payload": {
          "description": "All server-to-client messages",
          "oneOf": [
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[0]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[1]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[2]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[3]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[4]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[5]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[6]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[7]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[8]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[9]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[10]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[11]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[12]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[13]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[14]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[15]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[16]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[17]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[18]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[19]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[20]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[21]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[22]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[23]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[24]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[25]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[26]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[27]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[28]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[29]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[30]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[31]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[32]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[33]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[34]"
          ],
          "title": "ServerMessage",
          "x-parser-schema-id": "<anonymous-schema-446>"
        },
        "title": "payment.created",
        "x-parser-unique-object-id": "payment.created"
      },
      "payment.list": {
        "contentType": "application/json",
        "name": "payment.list",
        "payload": {
          "description": "All client-to-server messages",
          "oneOf": [
            "$ref:$.components.messages.auth.login.payload.oneOf[0]",
            "$ref:$.components.messages.auth.login.payload.oneOf[1]",
            "$ref:$.components.messages.auth.login.payload.oneOf[2]",
            "$ref:$.components.messages.auth.login.payload.oneOf[3]",
            "$ref:$.components.messages.auth.login.payload.oneOf[4]",
            "$ref:$.components.messages.auth.login.payload.oneOf[5]",
            "$ref:$.components.messages.auth.login.payload.oneOf[6]",
            "$ref:$.components.messages.auth.login.payload.oneOf[7]",
            "$ref:$.components.messages.auth.login.payload.oneOf[8]",
            "$ref:$.components.messages.auth.login.payload.oneOf[9]",
            "$ref:$.components.messages.auth.login.payload.oneOf[10]",
            "$ref:$.components.messages.auth.login.payload.oneOf[11]",
            "$ref:$.components.messages.auth.login.payload.oneOf[12]",
            "$ref:$.components.messages.auth.login.payload.oneOf[13]",
            "$ref:$.components.messages.auth.login.payload.oneOf[14]",
            "$ref:$.components.messages.auth.login.payload.oneOf[15]",
            "$ref:$.components.messages.auth.login.payload.oneOf[16]",
            "$ref:$.components.messages.auth.login.payload.oneOf[17]",
            "$ref:$.components.messages.auth.login.payload.oneOf[18]",
            "$ref:$.components.messages.auth.login.payload.oneOf[19]",
            "$ref:$.components.messages.auth.login.payload.oneOf[20]",
            "$ref:$.components.messages.auth.login.payload.oneOf[21]",
            "$ref:$.components.messages.auth.login.payload.oneOf[22]",
            "$ref:$.components.messages.auth.login.payload.oneOf[23]",
            "$ref:$.components.messages.auth.login.payload.oneOf[24]",
            "$ref:$.components.messages.auth.login.payload.oneOf[25]",
            "$ref:$.components.messages.auth.login.payload.oneOf[26]",
            "$ref:$.components.messages.auth.login.payload.oneOf[27]",
            "$ref:$.components.messages.auth.login.payload.oneOf[28]",
            "$ref:$.components.messages.auth.login.payload.oneOf[29]",
            "$ref:$.components.messages.auth.login.payload.oneOf[30]",
            "$ref:$.components.messages.auth.login.payload.oneOf[31]"
          ],
          "title": "ClientMessage",
          "x-parser-schema-id": "<anonymous-schema-447>"
        },
        "title": "payment.list",
        "x-parser-unique-object-id": "payment.list"
      },
      "payment.list.response": {
        "contentType": "application/json",
        "name": "payment.list.response",
        "payload": {
          "description": "All server-to-client messages",
          "oneOf": [
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[0]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[1]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[2]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[3]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[4]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[5]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[6]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[7]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[8]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[9]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[10]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[11]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[12]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[13]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[14]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[15]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[16]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[17]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[18]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[19]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[20]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[21]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[22]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[23]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[24]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[25]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[26]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[27]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[28]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[29]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[30]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[31]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[32]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[33]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[34]"
          ],
          "title": "ServerMessage",
          "x-parser-schema-id": "<anonymous-schema-448>"
        },
        "title": "payment.list.response",
        "x-parser-unique-object-id": "payment.list.response"
      },
      "payment.verified": {
        "contentType": "application/json",
        "name": "payment.verified",
        "payload": {
          "description": "All server-to-client messages",
          "oneOf": [
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[0]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[1]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[2]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[3]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[4]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[5]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[6]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[7]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[8]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[9]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[10]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[11]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[12]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[13]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[14]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[15]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[16]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[17]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[18]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[19]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[20]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[21]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[22]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[23]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[24]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[25]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[26]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[27]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[28]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[29]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[30]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[31]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[32]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[33]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[34]"
          ],
          "title": "ServerMessage",
          "x-parser-schema-id": "<anonymous-schema-449>"
        },
        "title": "payment.verified",
        "x-parser-unique-object-id": "payment.verified"
      },
      "payment.verify": {
        "contentType": "application/json",
        "name": "payment.verify",
        "payload": {
          "description": "All client-to-server messages",
          "oneOf": [
            "$ref:$.components.messages.auth.login.payload.oneOf[0]",
            "$ref:$.components.messages.auth.login.payload.oneOf[1]",
            "$ref:$.components.messages.auth.login.payload.oneOf[2]",
            "$ref:$.components.messages.auth.login.payload.oneOf[3]",
            "$ref:$.components.messages.auth.login.payload.oneOf[4]",
            "$ref:$.components.messages.auth.login.payload.oneOf[5]",
            "$ref:$.components.messages.auth.login.payload.oneOf[6]",
            "$ref:$.components.messages.auth.login.payload.oneOf[7]",
            "$ref:$.components.messages.auth.login.payload.oneOf[8]",
            "$ref:$.components.messages.auth.login.payload.oneOf[9]",
            "$ref:$.components.messages.auth.login.payload.oneOf[10]",
            "$ref:$.components.messages.auth.login.payload.oneOf[11]",
            "$ref:$.components.messages.auth.login.payload.oneOf[12]",
            "$ref:$.components.messages.auth.login.payload.oneOf[13]",
            "$ref:$.components.messages.auth.login.payload.oneOf[14]",
            "$ref:$.components.messages.auth.login.payload.oneOf[15]",
            "$ref:$.components.messages.auth.login.payload.oneOf[16]",
            "$ref:$.components.messages.auth.login.payload.oneOf[17]",
            "$ref:$.components.messages.auth.login.payload.oneOf[18]",
            "$ref:$.components.messages.auth.login.payload.oneOf[19]",
            "$ref:$.components.messages.auth.login.payload.oneOf[20]",
            "$ref:$.components.messages.auth.login.payload.oneOf[21]",
            "$ref:$.components.messages.auth.login.payload.oneOf[22]",
            "$ref:$.components.messages.auth.login.payload.oneOf[23]",
            "$ref:$.components.messages.auth.login.payload.oneOf[24]",
            "$ref:$.components.messages.auth.login.payload.oneOf[25]",
            "$ref:$.components.messages.auth.login.payload.oneOf[26]",
            "$ref:$.components.messages.auth.login.payload.oneOf[27]",
            "$ref:$.components.messages.auth.login.payload.oneOf[28]",
            "$ref:$.components.messages.auth.login.payload.oneOf[29]",
            "$ref:$.components.messages.auth.login.payload.oneOf[30]",
            "$ref:$.components.messages.auth.login.payload.oneOf[31]"
          ],
          "title": "ClientMessage",
          "x-parser-schema-id": "<anonymous-schema-450>"
        },
        "title": "payment.verify",
        "x-parser-unique-object-id": "payment.verify"
      },
      "product.create": {
        "contentType": "application/json",
        "name": "product.create",
        "payload": {
          "description": "All client-to-server messages",
          "oneOf": [
            "$ref:$.components.messages.auth.login.payload.oneOf[0]",
            "$ref:$.components.messages.auth.login.payload.oneOf[1]",
            "$ref:$.components.messages.auth.login.payload.oneOf[2]",
            "$ref:$.components.messages.auth.login.payload.oneOf[3]",
            "$ref:$.components.messages.auth.login.payload.oneOf[4]",
            "$ref:$.components.messages.auth.login.payload.oneOf[5]",
            "$ref:$.components.messages.auth.login.payload.oneOf[6]",
            "$ref:$.components.messages.auth.login.payload.oneOf[7]",
            "$ref:$.components.messages.auth.login.payload.oneOf[8]",
            "$ref:$.components.messages.auth.login.payload.oneOf[9]",
            "$ref:$.components.messages.auth.login.payload.oneOf[10]",
            "$ref:$.components.messages.auth.login.payload.oneOf[11]",
            "$ref:$.components.messages.auth.login.payload.oneOf[12]",
            "$ref:$.components.messages.auth.login.payload.oneOf[13]",
            "$ref:$.components.messages.auth.login.payload.oneOf[14]",
            "$ref:$.components.messages.auth.login.payload.oneOf[15]",
            "$ref:$.components.messages.auth.login.payload.oneOf[16]",
            "$ref:$.components.messages.auth.login.payload.oneOf[17]",
            "$ref:$.components.messages.auth.login.payload.oneOf[18]",
            "$ref:$.components.messages.auth.login.payload.oneOf[19]",
            "$ref:$.components.messages.auth.login.payload.oneOf[20]",
            "$ref:$.components.messages.auth.login.payload.oneOf[21]",
            "$ref:$.components.messages.auth.login.payload.oneOf[22]",
            "$ref:$.components.messages.auth.login.payload.oneOf[23]",
            "$ref:$.components.messages.auth.login.payload.oneOf[24]",
            "$ref:$.components.messages.auth.login.payload.oneOf[25]",
            "$ref:$.components.messages.auth.login.payload.oneOf[26]",
            "$ref:$.components.messages.auth.login.payload.oneOf[27]",
            "$ref:$.components.messages.auth.login.payload.oneOf[28]",
            "$ref:$.components.messages.auth.login.payload.oneOf[29]",
            "$ref:$.components.messages.auth.login.payload.oneOf[30]",
            "$ref:$.components.messages.auth.login.payload.oneOf[31]"
          ],
          "title": "ClientMessage",
          "x-parser-schema-id": "<anonymous-schema-451>"
        },
        "title": "product.create",
        "x-parser-unique-object-id": "product.create"
      },
      "product.created": {
        "contentType": "application/json",
        "name": "product.created",
        "payload": {
          "description": "All server-to-client messages",
          "oneOf": [
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[0]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[1]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[2]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[3]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[4]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[5]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[6]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[7]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[8]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[9]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[10]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[11]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[12]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[13]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[14]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[15]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[16]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[17]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[18]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[19]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[20]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[21]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[22]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[23]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[24]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[25]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[26]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[27]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[28]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[29]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[30]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[31]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[32]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[33]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[34]"
          ],
          "title": "ServerMessage",
          "x-parser-schema-id": "<anonymous-schema-452>"
        },
        "title": "product.created",
        "x-parser-unique-object-id": "product.created"
      },
      "product.delete": {
        "contentType": "application/json",
        "name": "product.delete",
        "payload": {
          "description": "All client-to-server messages",
          "oneOf": [
            "$ref:$.components.messages.auth.login.payload.oneOf[0]",
            "$ref:$.components.messages.auth.login.payload.oneOf[1]",
            "$ref:$.components.messages.auth.login.payload.oneOf[2]",
            "$ref:$.components.messages.auth.login.payload.oneOf[3]",
            "$ref:$.components.messages.auth.login.payload.oneOf[4]",
            "$ref:$.components.messages.auth.login.payload.oneOf[5]",
            "$ref:$.components.messages.auth.login.payload.oneOf[6]",
            "$ref:$.components.messages.auth.login.payload.oneOf[7]",
            "$ref:$.components.messages.auth.login.payload.oneOf[8]",
            "$ref:$.components.messages.auth.login.payload.oneOf[9]",
            "$ref:$.components.messages.auth.login.payload.oneOf[10]",
            "$ref:$.components.messages.auth.login.payload.oneOf[11]",
            "$ref:$.components.messages.auth.login.payload.oneOf[12]",
            "$ref:$.components.messages.auth.login.payload.oneOf[13]",
            "$ref:$.components.messages.auth.login.payload.oneOf[14]",
            "$ref:$.components.messages.auth.login.payload.oneOf[15]",
            "$ref:$.components.messages.auth.login.payload.oneOf[16]",
            "$ref:$.components.messages.auth.login.payload.oneOf[17]",
            "$ref:$.components.messages.auth.login.payload.oneOf[18]",
            "$ref:$.components.messages.auth.login.payload.oneOf[19]",
            "$ref:$.components.messages.auth.login.payload.oneOf[20]",
            "$ref:$.components.messages.auth.login.payload.oneOf[21]",
            "$ref:$.components.messages.auth.login.payload.oneOf[22]",
            "$ref:$.components.messages.auth.login.payload.oneOf[23]",
            "$ref:$.components.messages.auth.login.payload.oneOf[24]",
            "$ref:$.components.messages.auth.login.payload.oneOf[25]",
            "$ref:$.components.messages.auth.login.payload.oneOf[26]",
            "$ref:$.components.messages.auth.login.payload.oneOf[27]",
            "$ref:$.components.messages.auth.login.payload.oneOf[28]",
            "$ref:$.components.messages.auth.login.payload.oneOf[29]",
            "$ref:$.components.messages.auth.login.payload.oneOf[30]",
            "$ref:$.components.messages.auth.login.payload.oneOf[31]"
          ],
          "title": "ClientMessage",
          "x-parser-schema-id": "<anonymous-schema-453>"
        },
        "title": "product.delete",
        "x-parser-unique-object-id": "product.delete"
      },
      "product.deleted": {
        "contentType": "application/json",
        "name": "product.deleted",
        "payload": {
          "description": "All server-to-client messages",
          "oneOf": [
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[0]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[1]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[2]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[3]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[4]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[5]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[6]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[7]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[8]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[9]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[10]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[11]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[12]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[13]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[14]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[15]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[16]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[17]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[18]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[19]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[20]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[21]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[22]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[23]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[24]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[25]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[26]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[27]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[28]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[29]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[30]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[31]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[32]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[33]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[34]"
          ],
          "title": "ServerMessage",
          "x-parser-schema-id": "<anonymous-schema-454>"
        },
        "title": "product.deleted",
        "x-parser-unique-object-id": "product.deleted"
      },
      "product.list": {
        "contentType": "application/json",
        "name": "product.list",
        "payload": {
          "description": "All client-to-server messages",
          "oneOf": [
            "$ref:$.components.messages.auth.login.payload.oneOf[0]",
            "$ref:$.components.messages.auth.login.payload.oneOf[1]",
            "$ref:$.components.messages.auth.login.payload.oneOf[2]",
            "$ref:$.components.messages.auth.login.payload.oneOf[3]",
            "$ref:$.components.messages.auth.login.payload.oneOf[4]",
            "$ref:$.components.messages.auth.login.payload.oneOf[5]",
            "$ref:$.components.messages.auth.login.payload.oneOf[6]",
            "$ref:$.components.messages.auth.login.payload.oneOf[7]",
            "$ref:$.components.messages.auth.login.payload.oneOf[8]",
            "$ref:$.components.messages.auth.login.payload.oneOf[9]",
            "$ref:$.components.messages.auth.login.payload.oneOf[10]",
            "$ref:$.components.messages.auth.login.payload.oneOf[11]",
            "$ref:$.components.messages.auth.login.payload.oneOf[12]",
            "$ref:$.components.messages.auth.login.payload.oneOf[13]",
            "$ref:$.components.messages.auth.login.payload.oneOf[14]",
            "$ref:$.components.messages.auth.login.payload.oneOf[15]",
            "$ref:$.components.messages.auth.login.payload.oneOf[16]",
            "$ref:$.components.messages.auth.login.payload.oneOf[17]",
            "$ref:$.components.messages.auth.login.payload.oneOf[18]",
            "$ref:$.components.messages.auth.login.payload.oneOf[19]",
            "$ref:$.components.messages.auth.login.payload.oneOf[20]",
            "$ref:$.components.messages.auth.login.payload.oneOf[21]",
            "$ref:$.components.messages.auth.login.payload.oneOf[22]",
            "$ref:$.components.messages.auth.login.payload.oneOf[23]",
            "$ref:$.components.messages.auth.login.payload.oneOf[24]",
            "$ref:$.components.messages.auth.login.payload.oneOf[25]",
            "$ref:$.components.messages.auth.login.payload.oneOf[26]",
            "$ref:$.components.messages.auth.login.payload.oneOf[27]",
            "$ref:$.components.messages.auth.login.payload.oneOf[28]",
            "$ref:$.components.messages.auth.login.payload.oneOf[29]",
            "$ref:$.components.messages.auth.login.payload.oneOf[30]",
            "$ref:$.components.messages.auth.login.payload.oneOf[31]"
          ],
          "title": "ClientMessage",
          "x-parser-schema-id": "<anonymous-schema-455>"
        },
        "title": "product.list",
        "x-parser-unique-object-id": "product.list"
      },
      "product.list.response": {
        "contentType": "application/json",
        "name": "product.list.response",
        "payload": {
          "description": "All server-to-client messages",
          "oneOf": [
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[0]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[1]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[2]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[3]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[4]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[5]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[6]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[7]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[8]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[9]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[10]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[11]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[12]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[13]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[14]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[15]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[16]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[17]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[18]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[19]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[20]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[21]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[22]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[23]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[24]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[25]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[26]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[27]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[28]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[29]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[30]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[31]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[32]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[33]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[34]"
          ],
          "title": "ServerMessage",
          "x-parser-schema-id": "<anonymous-schema-456>"
        },
        "title": "product.list.response",
        "x-parser-unique-object-id": "product.list.response"
      },
      "product.modified": {
        "contentType": "application/json",
        "name": "product.modified",
        "payload": {
          "description": "All server-to-client messages",
          "oneOf": [
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[0]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[1]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[2]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[3]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[4]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[5]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[6]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[7]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[8]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[9]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[10]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[11]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[12]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[13]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[14]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[15]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[16]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[17]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[18]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[19]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[20]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[21]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[22]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[23]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[24]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[25]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[26]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[27]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[28]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[29]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[30]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[31]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[32]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[33]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[34]"
          ],
          "title": "ServerMessage",
          "x-parser-schema-id": "<anonymous-schema-457>"
        },
        "title": "product.modified",
        "x-parser-unique-object-id": "product.modified"
      },
      "product.modify": {
        "contentType": "application/json",
        "name": "product.modify",
        "payload": {
          "description": "All client-to-server messages",
          "oneOf": [
            "$ref:$.components.messages.auth.login.payload.oneOf[0]",
            "$ref:$.components.messages.auth.login.payload.oneOf[1]",
            "$ref:$.components.messages.auth.login.payload.oneOf[2]",
            "$ref:$.components.messages.auth.login.payload.oneOf[3]",
            "$ref:$.components.messages.auth.login.payload.oneOf[4]",
            "$ref:$.components.messages.auth.login.payload.oneOf[5]",
            "$ref:$.components.messages.auth.login.payload.oneOf[6]",
            "$ref:$.components.messages.auth.login.payload.oneOf[7]",
            "$ref:$.components.messages.auth.login.payload.oneOf[8]",
            "$ref:$.components.messages.auth.login.payload.oneOf[9]",
            "$ref:$.components.messages.auth.login.payload.oneOf[10]",
            "$ref:$.components.messages.auth.login.payload.oneOf[11]",
            "$ref:$.components.messages.auth.login.payload.oneOf[12]",
            "$ref:$.components.messages.auth.login.payload.oneOf[13]",
            "$ref:$.components.messages.auth.login.payload.oneOf[14]",
            "$ref:$.components.messages.auth.login.payload.oneOf[15]",
            "$ref:$.components.messages.auth.login.payload.oneOf[16]",
            "$ref:$.components.messages.auth.login.payload.oneOf[17]",
            "$ref:$.components.messages.auth.login.payload.oneOf[18]",
            "$ref:$.components.messages.auth.login.payload.oneOf[19]",
            "$ref:$.components.messages.auth.login.payload.oneOf[20]",
            "$ref:$.components.messages.auth.login.payload.oneOf[21]",
            "$ref:$.components.messages.auth.login.payload.oneOf[22]",
            "$ref:$.components.messages.auth.login.payload.oneOf[23]",
            "$ref:$.components.messages.auth.login.payload.oneOf[24]",
            "$ref:$.components.messages.auth.login.payload.oneOf[25]",
            "$ref:$.components.messages.auth.login.payload.oneOf[26]",
            "$ref:$.components.messages.auth.login.payload.oneOf[27]",
            "$ref:$.components.messages.auth.login.payload.oneOf[28]",
            "$ref:$.components.messages.auth.login.payload.oneOf[29]",
            "$ref:$.components.messages.auth.login.payload.oneOf[30]",
            "$ref:$.components.messages.auth.login.payload.oneOf[31]"
          ],
          "title": "ClientMessage",
          "x-parser-schema-id": "<anonymous-schema-458>"
        },
        "title": "product.modify",
        "x-parser-unique-object-id": "product.modify"
      },
      "review.create": {
        "contentType": "application/json",
        "name": "review.create",
        "payload": {
          "description": "All client-to-server messages",
          "oneOf": [
            "$ref:$.components.messages.auth.login.payload.oneOf[0]",
            "$ref:$.components.messages.auth.login.payload.oneOf[1]",
            "$ref:$.components.messages.auth.login.payload.oneOf[2]",
            "$ref:$.components.messages.auth.login.payload.oneOf[3]",
            "$ref:$.components.messages.auth.login.payload.oneOf[4]",
            "$ref:$.components.messages.auth.login.payload.oneOf[5]",
            "$ref:$.components.messages.auth.login.payload.oneOf[6]",
            "$ref:$.components.messages.auth.login.payload.oneOf[7]",
            "$ref:$.components.messages.auth.login.payload.oneOf[8]",
            "$ref:$.components.messages.auth.login.payload.oneOf[9]",
            "$ref:$.components.messages.auth.login.payload.oneOf[10]",
            "$ref:$.components.messages.auth.login.payload.oneOf[11]",
            "$ref:$.components.messages.auth.login.payload.oneOf[12]",
            "$ref:$.components.messages.auth.login.payload.oneOf[13]",
            "$ref:$.components.messages.auth.login.payload.oneOf[14]",
            "$ref:$.components.messages.auth.login.payload.oneOf[15]",
            "$ref:$.components.messages.auth.login.payload.oneOf[16]",
            "$ref:$.components.messages.auth.login.payload.oneOf[17]",
            "$ref:$.components.messages.auth.login.payload.oneOf[18]",
            "$ref:$.components.messages.auth.login.payload.oneOf[19]",
            "$ref:$.components.messages.auth.login.payload.oneOf[20]",
            "$ref:$.components.messages.auth.login.payload.oneOf[21]",
            "$ref:$.components.messages.auth.login.payload.oneOf[22]",
            "$ref:$.components.messages.auth.login.payload.oneOf[23]",
            "$ref:$.components.messages.auth.login.payload.oneOf[24]",
            "$ref:$.components.messages.auth.login.payload.oneOf[25]",
            "$ref:$.components.messages.auth.login.payload.oneOf[26]",
            "$ref:$.components.messages.auth.login.payload.oneOf[27]",
            "$ref:$.components.messages.auth.login.payload.oneOf[28]",
            "$ref:$.components.messages.auth.login.payload.oneOf[29]",
            "$ref:$.components.messages.auth.login.payload.oneOf[30]",
            "$ref:$.components.messages.auth.login.payload.oneOf[31]"
          ],
          "title": "ClientMessage",
          "x-parser-schema-id": "<anonymous-schema-459>"
        },
        "title": "review.create",
        "x-parser-unique-object-id": "review.create"
      },
      "review.created": {
        "contentType": "application/json",
        "name": "review.created",
        "payload": {
          "description": "All server-to-client messages",
          "oneOf": [
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[0]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[1]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[2]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[3]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[4]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[5]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[6]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[7]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[8]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[9]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[10]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[11]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[12]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[13]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[14]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[15]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[16]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[17]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[18]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[19]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[20]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[21]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[22]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[23]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[24]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[25]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[26]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[27]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[28]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[29]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[30]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[31]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[32]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[33]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[34]"
          ],
          "title": "ServerMessage",
          "x-parser-schema-id": "<anonymous-schema-460>"
        },
        "title": "review.created",
        "x-parser-unique-object-id": "review.created"
      },
      "review.list": {
        "contentType": "application/json",
        "name": "review.list",
        "payload": {
          "description": "All client-to-server messages",
          "oneOf": [
            "$ref:$.components.messages.auth.login.payload.oneOf[0]",
            "$ref:$.components.messages.auth.login.payload.oneOf[1]",
            "$ref:$.components.messages.auth.login.payload.oneOf[2]",
            "$ref:$.components.messages.auth.login.payload.oneOf[3]",
            "$ref:$.components.messages.auth.login.payload.oneOf[4]",
            "$ref:$.components.messages.auth.login.payload.oneOf[5]",
            "$ref:$.components.messages.auth.login.payload.oneOf[6]",
            "$ref:$.components.messages.auth.login.payload.oneOf[7]",
            "$ref:$.components.messages.auth.login.payload.oneOf[8]",
            "$ref:$.components.messages.auth.login.payload.oneOf[9]",
            "$ref:$.components.messages.auth.login.payload.oneOf[10]",
            "$ref:$.components.messages.auth.login.payload.oneOf[11]",
            "$ref:$.components.messages.auth.login.payload.oneOf[12]",
            "$ref:$.components.messages.auth.login.payload.oneOf[13]",
            "$ref:$.components.messages.auth.login.payload.oneOf[14]",
            "$ref:$.components.messages.auth.login.payload.oneOf[15]",
            "$ref:$.components.messages.auth.login.payload.oneOf[16]",
            "$ref:$.components.messages.auth.login.payload.oneOf[17]",
            "$ref:$.components.messages.auth.login.payload.oneOf[18]",
            "$ref:$.components.messages.auth.login.payload.oneOf[19]",
            "$ref:$.components.messages.auth.login.payload.oneOf[20]",
            "$ref:$.components.messages.auth.login.payload.oneOf[21]",
            "$ref:$.components.messages.auth.login.payload.oneOf[22]",
            "$ref:$.components.messages.auth.login.payload.oneOf[23]",
            "$ref:$.components.messages.auth.login.payload.oneOf[24]",
            "$ref:$.components.messages.auth.login.payload.oneOf[25]",
            "$ref:$.components.messages.auth.login.payload.oneOf[26]",
            "$ref:$.components.messages.auth.login.payload.oneOf[27]",
            "$ref:$.components.messages.auth.login.payload.oneOf[28]",
            "$ref:$.components.messages.auth.login.payload.oneOf[29]",
            "$ref:$.components.messages.auth.login.payload.oneOf[30]",
            "$ref:$.components.messages.auth.login.payload.oneOf[31]"
          ],
          "title": "ClientMessage",
          "x-parser-schema-id": "<anonymous-schema-461>"
        },
        "title": "review.list",
        "x-parser-unique-object-id": "review.list"
      },
      "review.list.response": {
        "contentType": "application/json",
        "name": "review.list.response",
        "payload": {
          "description": "All server-to-client messages",
          "oneOf": [
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[0]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[1]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[2]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[3]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[4]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[5]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[6]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[7]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[8]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[9]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[10]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[11]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[12]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[13]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[14]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[15]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[16]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[17]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[18]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[19]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[20]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[21]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[22]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[23]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[24]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[25]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[26]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[27]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[28]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[29]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[30]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[31]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[32]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[33]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[34]"
          ],
          "title": "ServerMessage",
          "x-parser-schema-id": "<anonymous-schema-462>"
        },
        "title": "review.list.response",
        "x-parser-unique-object-id": "review.list.response"
      },
      "server.ping": {
        "contentType": "application/json",
        "name": "server.ping",
        "payload": {
          "description": "All server-to-client messages",
          "oneOf": [
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[0]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[1]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[2]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[3]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[4]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[5]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[6]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[7]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[8]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[9]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[10]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[11]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[12]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[13]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[14]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[15]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[16]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[17]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[18]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[19]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[20]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[21]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[22]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[23]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[24]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[25]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[26]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[27]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[28]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[29]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[30]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[31]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[32]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[33]",
            "$ref:$.components.messages.auth.logout.response.payload.oneOf[34]"
          ],
          "title": "ServerMessage",
          "x-parser-schema-id": "<anonymous-schema-463>"
        },
        "title": "server.ping",
        "x-parser-unique-object-id": "server.ping"
      }
    },
    "schemas": {
      "AuthLoginRequest": "$ref:$.components.messages.auth.login.payload.oneOf[0]",
      "AuthLogoutRequest": "$ref:$.components.messages.auth.login.payload.oneOf[1]",
      "AuthLogoutResponse": "$ref:$.components.messages.auth.logout.response.payload.oneOf[1]",
      "AuthResponse": "$ref:$.components.messages.auth.logout.response.payload.oneOf[0]",
      "ChannelCreateRequest": "$ref:$.components.messages.auth.login.payload.oneOf[13]",
      "ChannelCreatedNotification": "$ref:$.components.messages.auth.logout.response.payload.oneOf[12]",
      "ChannelDeleteRequest": "$ref:$.components.messages.auth.login.payload.oneOf[15]",
      "ChannelDeletedNotification": "$ref:$.components.messages.auth.logout.response.payload.oneOf[14]",
      "ChannelEditRequest": "$ref:$.components.messages.auth.login.payload.oneOf[14]",
      "ChannelEditedNotification": "$ref:$.components.messages.auth.logout.response.payload.oneOf[13]",
      "ChannelInfo": "$ref:$.components.messages.auth.logout.response.payload.oneOf[10].properties.channels.items",
      "ChannelInfoRequest": "$ref:$.components.messages.auth.login.payload.oneOf[12]",
      "ChannelInfoResponse": "$ref:$.components.messages.auth.logout.response.payload.oneOf[11]",
      "ChannelListRequest": "$ref:$.components.messages.auth.login.payload.oneOf[11]",
      "ChannelListResponse": "$ref:$.components.messages.auth.logout.response.payload.oneOf[10]",
      "ChannelType": "$ref:$.components.messages.auth.login.payload.oneOf[13].properties.channel_type",
      "ConnectionStatus": "$ref:$.components.messages.auth.logout.response.payload.oneOf[34].properties.status",
      "ConnectionStatusMessage": "$ref:$.components.messages.auth.logout.response.payload.oneOf[34]",
      "DeviceInfo": "$ref:$.components.messages.auth.logout.response.payload.oneOf[0].properties.device_info.anyOf[0]",
      "ErrorResponse": "$ref:$.components.messages.auth.logout.response.payload.oneOf[32]",
      "HostingCloseRequest": "$ref:$.components.messages.auth.login.payload.oneOf[6]",
      "HostingDetails": "$ref:$.components.messages.auth.logout.response.payload.oneOf[3].properties.hosting.anyOf[0]",
      "HostingInitRequest": "$ref:$.components.messages.auth.login.payload.oneOf[2]",
      "HostingInitResponse": "$ref:$.components.messages.auth.logout.response.payload.oneOf[2]",
      "HostingListRequest": "$ref:$.components.messages.auth.login.payload.oneOf[5]",
      "HostingListResponse": "$ref:$.components.messages.auth.logout.response.payload.oneOf[5]",
      "HostingSelectRequest": "$ref:$.components.messages.auth.login.payload.oneOf[4]",
      "HostingSelectResponse": "$ref:$.components.messages.auth.logout.response.payload.oneOf[4]",
      "HostingShowRequest": "$ref:$.components.messages.auth.login.payload.oneOf[3]",
      "HostingShowResponse": "$ref:$.components.messages.auth.logout.response.payload.oneOf[3]",
      "MemberBanRequest": "$ref:$.components.messages.auth.login.payload.oneOf[10]",
      "MemberBannedNotification": "$ref:$.components.messages.auth.logout.response.payload.oneOf[9]",
      "MemberInfo": "$ref:$.components.messages.auth.logout.response.payload.oneOf[6].properties.members.items",
      "MemberInfoRequest": "$ref:$.components.messages.auth.login.payload.oneOf[8]",
      "MemberInfoResponse": "$ref:$.components.messages.auth.logout.response.payload.oneOf[7]",
      "MemberKickRequest": "$ref:$.components.messages.auth.login.payload.oneOf[9]",
      "MemberKickedNotification": "$ref:$.components.messages.auth.logout.response.payload.oneOf[8]",
      "MemberListRequest": "$ref:$.components.messages.auth.login.payload.oneOf[7]",
      "MemberListResponse": "$ref:$.components.messages.auth.logout.response.payload.oneOf[6]",
      "MemberType": "$ref:$.components.messages.auth.logout.response.payload.oneOf[6].properties.members.items.properties.member_type",
      "MessageAttachment": "$ref:$.components.messages.auth.login.payload.oneOf[16].properties.attachments.items",
      "MessageData": "$ref:$.components.messages.auth.logout.response.payload.oneOf[15].properties.message.anyOf[0]",
      "MessageDeleteRequest": "$ref:$.components.messages.auth.login.payload.oneOf[19]",
      "MessageDeletedNotification": "$ref:$.components.messages.auth.logout.response.payload.oneOf[19]",
      "MessageEditRequest": "$ref:$.components.messages.auth.login.payload.oneOf[18]",
      "MessageEditedNotification": "$ref:$.components.messages.auth.logout.response.payload.oneOf[18]",
      "MessageListRequest": "$ref:$.components.messages.auth.login.payload.oneOf[17]",
      "MessageListResponse": "$ref:$.components.messages.auth.logout.response.payload.oneOf[16]",
      "MessageReaction": "$ref:$.components.messages.auth.logout.response.payload.oneOf[15].properties.message.anyOf[0].properties.reactions.items",
      "MessageReceivedNotification": "$ref:$.components.messages.auth.logout.response.payload.oneOf[17]",
      "MessageReplyRequest": "$ref:$.components.messages.auth.login.payload.oneOf[20]",
      "MessageSendRequest": "$ref:$.components.messages.auth.login.payload.oneOf[16]",
      "MessageSentResponse": "$ref:$.components.messages.auth.logout.response.payload.oneOf[15]",
      "OrderCreateRequest": "$ref:$.components.messages.auth.login.payload.oneOf[25]",
      "OrderCreatedResponse": "$ref:$.components.messages.auth.logout.response.payload.oneOf[24]",
      "OrderData": "$ref:$.components.messages.auth.logout.response.payload.oneOf[24].properties.order.anyOf[0]",
      "OrderItem": "$ref:$.components.messages.auth.logout.response.payload.oneOf[24].properties.order.anyOf[0].properties.items.items",
      "OrderListRequest": "$ref:$.components.messages.auth.login.payload.oneOf[26]",
      "OrderListResponse": "$ref:$.components.messages.auth.logout.response.payload.oneOf[25]",
      "OrderStatus": "$ref:$.components.messages.auth.login.payload.oneOf[26].properties.status.anyOf[0]",
      "OrderStatusUpdateNotification": "$ref:$.components.messages.auth.logout.response.payload.oneOf[26]",
      "PaymentCreateRequest": "$ref:$.components.messages.auth.login.payload.oneOf[27]",
      "PaymentCreatedResponse": "$ref:$.components.messages.auth.logout.response.payload.oneOf[27]",
      "PaymentData": "$ref:$.components.messages.auth.logout.response.payload.oneOf[27].properties.payment.anyOf[0]",
      "PaymentListRequest": "$ref:$.components.messages.auth.login.payload.oneOf[29]",
      "PaymentListResponse": "$ref:$.components.messages.auth.logout.response.payload.oneOf[29]",
      "PaymentMethod": "$ref:$.components.messages.auth.login.payload.oneOf[27].properties.payment_method",
      "PaymentStatus": "$ref:$.components.messages.auth.login.payload.oneOf[29].properties.status.anyOf[0]",
      "PaymentVerifiedResponse": "$ref:$.components.messages.auth.logout.response.payload.oneOf[28]",
      "PaymentVerifyRequest": "$ref:$.components.messages.auth.login.payload.oneOf[28]",
      "ProductCreateRequest": "$ref:$.components.messages.auth.login.payload.oneOf[21]",
      "ProductCreatedResponse": "$ref:$.components.messages.auth.logout.response.payload.oneOf[20]",
      "ProductData": "$ref:$.components.messages.auth.logout.response.payload.oneOf[20].properties.product.anyOf[0]",
      "ProductDeleteRequest": "$ref:$.components.messages.auth.login.payload.oneOf[24]",
      "ProductDeletedNotification": "$ref:$.components.messages.auth.logout.response.payload.oneOf[23]",
      "ProductListRequest": "$ref:$.components.messages.auth.login.payload.oneOf[22]",
      "ProductListResponse": "$ref:$.components.messages.auth.logout.response.payload.oneOf[21]",
      "ProductModifiedNotification": "$ref:$.components.messages.auth.logout.response.payload.oneOf[22]",
      "ProductModifyRequest": "$ref:$.components.messages.auth.login.payload.oneOf[23]",
      "ProductPrice": "$ref:$.components.messages.auth.login.payload.oneOf[21].properties.price",
      "RatingDistribution": "$ref:$.components.messages.auth.logout.response.payload.oneOf[31].properties.rating_distribution.anyOf[0]",
      "ReviewCreateRequest": "$ref:$.components.messages.auth.login.payload.oneOf[30]",
      "ReviewCreatedResponse": "$ref:$.components.messages.auth.logout.response.payload.oneOf[30]",
      "ReviewData": "$ref:$.components.messages.auth.logout.response.payload.oneOf[30].properties.review.anyOf[0]",
      "ReviewImage": "$ref:$.components.messages.auth.login.payload.oneOf[30].properties.images.items",
      "ReviewListRequest": "$ref:$.components.messages.auth.login.payload.oneOf[31]",
      "ReviewListResponse": "$ref:$.components.messages.auth.logout.response.payload.oneOf[31]",
      "ReviewReply": "$ref:$.components.messages.auth.logout.response.payload.oneOf[30].properties.review.anyOf[0].properties.reply.anyOf[0]",
      "ReviewSortBy": "$ref:$.components.messages.auth.login.payload.oneOf[31].properties.sort_by.anyOf[0]",
      "ServerPingMessage": "$ref:$.components.messages.auth.logout.response.payload.oneOf[33]",
      "ShippingAddress": "$ref:$.components.messages.auth.login.payload.oneOf[25].properties.shipping_address"
    }
  },
  "info": {
    "description": "Distributed e-commerce WebSocket protocol for the Dure platform. Supports real-time communication between clients, stores, and hosting servers.",
    "title": "Dure WebSocket API",
    "version": "1.0.0"
  },
  "servers": {
    "development": {
      "description": "Development WebSocket server for local testing",
      "host": "localhost:8443",
      "pathname": "/api/ws",
      "protocol": "wss"
    },
    "production": {
      "description": "Production WebSocket server for Dure stores",
      "host": "{domain}",
      "pathname": "/api/ws",
      "protocol": "wss",
      "variables": {
        "domain": {
          "description": "Store domain name (e.g., www.example.com, shop.mystore.com)",
          "examples": [
            "www.dure.com",
            "shop.example.com",
            "store.mydomain.com"
          ]
        }
      }
    }
  },
  "x-parser-spec-parsed": true,
  "x-parser-api-version": 3,
  "x-parser-spec-stringified": true
};
    const config = {"show":{"sidebar":true},"sidebar":{"showOperations":"byDefault"}};
    const appRoot = document.getElementById('root');
    AsyncApiStandalone.render(
        { schema, config, }, appRoot
    );
  