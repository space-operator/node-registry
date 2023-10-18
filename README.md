# node-registry

A place to store and test all the available nodes

## Use form builder to build form!

https://ginkgobioworks.github.io/react-json-schema-form-builder/
https://github.com/ginkgobioworks/react-json-schema-form-builder

## JSON to form schema

https://rjsf-team.github.io/react-jsonschema-form/

Need both JSONSchema and UISchema

## Node Definition

```
{
  "type": "native", // native | WASM | custom
  "data": {
    "node_definition_version": "0.1",
    "unique_id": "",
    "node_id": "mint-token",
    "version": "0.1",
    "display_name": "Mint Token",
    "description": "Create a token",
    "width": 250,
    "height": 0, // need to manually calculate height = 50 + 15 + max(sources.count, targets.count)*45
    "backgroundColorDark":"#000000" ,
    "backgroundColor": "#fff",
    "tags": [],
    "related_to_nodes": [
      {
        "id": "",
        "relationship": ""
      }
    ],
    "resources": {
      "source_code_url": "",
      "documentation_url": ""
    },
    "usage": {
      "license": "Apache-2.0",
      "license_url": "",
      "pricing": {
        "currency": "USDC",
        "purchase_price": 0,
        "price_per_run": 0,
        "custom": {
          "unit": "monthly",
          "value": "0"
        }
      }
    },
    "authors": [
      {
        "name": "Space Operator",
        "contact": ""
      }
    ],
    "design": {
      "width": 0,
      "height": 0,
      "icon_url": "",
      "backgroundColorDark": "#000000",
      "backgroundColor": "#fff"
    },
    "options": {}
  },
  // node outputs
  "sources": [
    {
      "name": "token_pubkey",
      "type": "string",
      "defaultValue": "", // might allow flow simulation without a run
      "tooltip": ""
    }
  ],
  // node inputs
  "targets": [
    {
      "name": "token_name",
      "type_bounds": [
        "string"
      ],
      "required": true,
      "defaultValue": "",
      "tooltip": "",
      "passthrough": true
    }
  ],
  // node inputs form - for static entry
  "targets_form.json_schema": {},
  "targets_form.ui_schema": {}
}
```
