"use client";
import UserPage from "../userpage";

export default function DebugLayout({ children }: { children: React.ReactNode }) {
  return <UserPage container>
    { children }
  </UserPage>
}