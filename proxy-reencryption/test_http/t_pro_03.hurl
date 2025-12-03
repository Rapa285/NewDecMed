POST {{PROXY_BASE_URL}}/nonce
{
    "iota_address": "0x74158cbda2fb5e9e4906416de1b349ff4972beb64a3e85a6d1509c32aa9b1621"
}
HTTP 200
[Asserts]
jsonpath "$.data" isString
