"use client"
import "./rankings.scss";
import { Container } from "react-bootstrap";

export default function RankingsLayout({ children }: { children: React.ReactNode }) {
  return <Container className="rankings-root">
    { children }
  </Container>
}