# Overview

This project is designed to render BMKG's weather radars on top of static map tiles. It includes
two packages:

- **radar\_worker**: Handles the image processing. The package contains both a `[[bin]]` target
  and a `lib.rs`.
- **discord**: A Discord bot with a command like `get_image` that uses the radar\_worker package.

To install any of them, you can use the following commands:

- For the Discord bot:
  ```
  cargo install --path discord
  ```
- For the radar worker CLI tool:
  ```
  cargo install --path radar_worker
  ```

The radar\_worker CLI tool comes with a `--help` function that is self-explanatory.

# Configuration Documentation

## Required Environment Variables

There are a few environment variables that must be set:

- **BMKG\_APIKEY**: API key for
  accessing [BMKG services](https://radar.bmkg.go.id/api/documentation/).
- **DISCORD\_TOKEN**: Token for the [Discord BOT](https://discord.dev).
- **THUNDERFOREST\_APIKEY**: API key for [Thunderforest services](https://thunderforest.com).

## Optional Configuration (Using a Proxy)

If you wish to use a proxy, set the following variable:

- **PROXY\_URL**: The URL of a [Cloudflare Worker](https://workers.cloudflare.com) proxy.

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

If you decide to use a proxy, the program will append the encoded target URL as a query parameter.
For example:

Original URL:

```
https://example.com
```

Resulted Proxy URL:

```
https://proxy.example.com?url=https%3A%2F%2Fexample.com
```

**Note**: This proxy configuration is entirely optional. It’s intended for some specific scenarios
where a normal fetch to the API may not be possible.
