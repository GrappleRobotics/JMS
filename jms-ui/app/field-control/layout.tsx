"use client"

import UserPage from "../userpage"

export default function FieldControlLayout({ children }: { children: React.ReactNode }) {
  return <UserPage>
    { children }
  </UserPage>
}