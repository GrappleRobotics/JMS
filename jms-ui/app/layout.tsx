import type { Metadata } from 'next'

import "./global.scss";
import RootLayoutInner from './layout_inner';

export const metadata: Metadata = {
  title: 'JMS',
  description: 'Another Alternative Field Management System for FRC',
  viewport: { initialScale: 1, width: "device-width" }
}

export default function RootLayout({
  children,
}: {
  children: React.ReactNode
}) {
  return (
    <html lang="en">
      <body>
        <RootLayoutInner>
          { children }
        </RootLayoutInner>
      </body>
    </html>
  )
}
