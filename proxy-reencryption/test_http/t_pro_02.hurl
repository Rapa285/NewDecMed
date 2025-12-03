POST {{PROXY_BASE_URL}}/nonce
{
    "iota_address": "abc"
}
HTTP 400
[Asserts]
jsonpath "$.error" == "Invalid patient IOTA address"
