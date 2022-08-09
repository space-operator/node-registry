# node-registry
A place to store and test all the available nodes


## JSON to form schema
https://rjsf-team.github.io/react-jsonschema-form/

Need both JSONSchema and UISchema

## Basic Example
```
{
    "version": 0.1,
    "node_id": "example",
    "extra.display_name": "HTTP Request",
    "type": "native", // native | custom | WASM
    "extra.width": null,
    "extra.height": null,
    // sources aka outputs
    "sources": [
        {
            "name": "response",
            "type": "JSON",
            "required": true,
            "defaultValue": "",
            "tooltip": ""
        },
        ...
    ],
    // targets aka inputs
    "targets": [
        {
            "name": "method",
            "type": "string",
            "type_bounds": [
                "string"
            ],
            "required": true,
            "defaultValue": "GET",
            "tooltip": "GET, POST, PATCH, etc.",
            "passthrough": false
        },
        ...
    ],
    "inputs.form.jsonschema": {}, // build a form on https://rjsf-team.github.io/react-jsonschema-form/
    "inputs.form.uischema": {}
}
```
