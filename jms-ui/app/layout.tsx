"use client";
import type { Metadata } from 'next'

import { WebsocketManagerComponent } from './support/ws-component';
import "./global.scss";

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
        <WebsocketManagerComponent>
          <main>
            { children }
          </main>
        </WebsocketManagerComponent>
      </body>
    </html>
  )
}
