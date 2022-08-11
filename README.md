# node-registry
A place to store and test all the available nodes


## JSON to form schema
https://rjsf-team.github.io/react-jsonschema-form/

Need both JSONSchema and UISchema

## Node Definition
```
{
  "type": "native", // native | WASM | custom
  "data": {
    "node_id": "mint-token",
    "version": "0.1",
    "display_name": "Mint Token",
    "description": "Create a token",
    "width": 250, 
    "height": 0, // need to manually calculate height = 50 + 15 + max(input.count, output.count)*45
    "backgroundColor": "#fff"
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
