"use client"
import { Button } from "react-bootstrap";
import UserPage from "./userpage";

export default function Home() {
  return (
    <UserPage container>
      <h3> Hello World </h3>
      <Button>Hello!</Button>
    </UserPage>
  )
}
