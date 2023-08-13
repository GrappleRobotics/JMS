/** @type {import('next').NextConfig} */
// Next won't let us export things that aren't the default in page.tsx, so we simply ignore those errors...
const nextConfig = {
  typescript: {
    ignoreBuildErrors: true,
  },
  output: "standalone"
}

module.exports = nextConfig
