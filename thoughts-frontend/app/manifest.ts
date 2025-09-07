import type { MetadataRoute } from 'next'
 
export default function manifest(): MetadataRoute.Manifest {
  return {
    name: 'Thoughts',
    short_name: 'Thoughts',
    description: 'A social network for sharing thoughts',
    start_url: '/',
    display: 'standalone',
    background_color: '#ffffff',
    theme_color: '#000000',
    icons: [
      {
        src: '/icon-192x192.webp',
        sizes: '192x192',
        type: 'image/webp',
      },
      {
        src: '/icon.avif',
        sizes: '512x512',
        type: 'image/avif',
      },
    ],
  }
}