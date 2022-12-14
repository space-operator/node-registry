{
    "type": "native",
    "data": {
        "node_id": "upgrade_authority",
        "version": "0.1",
        "display_name": "Upgrade Authority",
        "description": "",
        "width": 200,
        "height": 350,
        "backgroundColor": "#FFFF99"
    },
    "sources": [
        {
            "name": "signature",
            "type": "free",
            "defaultValue": null,
            "tooltip": ""
        }
    ],
    "targets": [
        {
            "name": "keypair",
            "type_bounds": [
                "keypair", "string"
            ],
            "required": false,
            "defaultValue": null,
            "tooltip": "",
            "passthrough": false
        },
        {
            "name": "payer",
            "type_bounds": [
                "keypair", "string"
            ],
            "required": false,
            "defaultValue": null,
            "tooltip": "",
            "passthrough": false
        }
    ],
    "targets_form.json_schema": {},
    "targets_form.ui_schema": {}
}