import type { Metadata } from 'next'
import ThemeRegistry from './ThemeRegistry';

import '@fontsource/roboto/300.css';
import '@fontsource/roboto/400.css';
import '@fontsource/roboto/500.css';
import '@fontsource/roboto/700.css';

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
          <ThemeRegistry options={{ key: 'mui' }}>{ children }</ThemeRegistry>
        </WebsocketManagerComponent>
      </body>
    </html>
  )
}
