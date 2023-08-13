"use client"

import ErrorProvider from "./support/errors"
import { WebsocketManagerComponent } from "./support/ws-component"

export default function RootLayoutInner({
  children,
}: {
  children: React.ReactNode
}) {
  return (
    <WebsocketManagerComponent>
      <ErrorProvider>
        <main>
          { children }
        </main>
      </ErrorProvider>
    </WebsocketManagerComponent>
  )
}
