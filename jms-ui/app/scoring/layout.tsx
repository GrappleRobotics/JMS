"use client"

import "./scoring.scss";
import UserPage from "../userpage"

export default function FieldControlLayout({ children }: { children: React.ReactNode }) {
  return <UserPage>
    { children }
  </UserPage>
}