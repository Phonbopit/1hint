export const proxyUsageExample = `// Replace your 1inch API calls with:
const ONEINCH_BASE_URL = '{PROXY_URL}';

// Example: Get swap quote
fetch(\`\${ONEINCH_BASE_URL}/v5.0/1/quote\`, {
  method: 'GET',
  // No need for API key or CORS headers!
})
.then(response => response.json())
.then(data => console.log(data));

// For production, change to:
// const ONEINCH_BASE_URL = 'https://api.1inch.io';
// (and handle API key properly on backend)`
