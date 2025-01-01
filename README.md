# Configuration Documentation

## Required Environment Variables

To run this program, the following environment variables must be set:

- **BMKG_APIKEY**: API key for
  accessing [BMKG services](https://radar.bmkg.go.id/api/documentation/).
- **DISCORD_TOKEN**: Token for the [Discord BOT](https://discord.dev).
- **THUNDERFOREST_APIKEY**: API key for [Thunderforest services](https://thunderforest.com).

## Optional Configuration (Using a Proxy)

If you wish to use a proxy, set the following variable:

- **PROXY_URL**: The URL of a Cloudflare Worker proxy.

### Proxy URL Example

Your `PROXY_URL` should point to a Cloudflare Worker proxy, for example:

```
https://proxy.example.com
```

### Quick Cloudflare Worker Configuration

You can do a quick setup by using this script:

```javascript
export default {
    async fetch(request, env, ctx) {
        const url = new URL(request.url);
        const targetUrl = url.searchParams.get('url');

        if (!targetUrl) {
            return new Response('Missing "url" query parameter', {status: 400});
        }

        try {
            const response = await fetch(decodeURIComponent(targetUrl), {
                method: request.method,
                headers: request.headers,
                body: request.method === 'POST' ? await request.text() : null
            });

            const proxyResponse = new Response(response.body, response);

            return proxyResponse;
        } catch (error) {
            return new Response('Error fetching the target URL', {status: 500});
        }
    },
};
```

### How It Works

The program will use the proxy by appending the encoded target URL as a query parameter. For
example:

Original URL:

```
https://example.com
```

Resulted Proxy URL:

```
https://proxy.example.com?url=https%3A%2F%2Fexample.com
```

