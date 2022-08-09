import { REPORTS } from "paths";
import React from "react";
import { Button, Col, Container, Row } from "react-bootstrap";

export const REPORT_MAP = [
  { title: "Teams", paths: [ { title: "Team Report",  path: "teams", variant: "info" } ] },
  { title: "Rankings", paths: [ { title: "Quals Rankings",  path: "rankings", variant: "primary" } ] },
  { title: "Awards", paths: [ { title: "Awards Report",  path: "awards", variant: "success" } ] },

  { 
    title: "WPA Keys (FTA ONLY)", 
    fta: true,
    paths: [ 
      { title: "PDF",  path: "wpa/pdf", variant: "danger" }, 
      { title: "CSV", path: "wpa/csv", variant: "warning" } 
    ] 
  },

  {
    title: "Quals Match Schedules", paths: [
      { title: "Quals", path: "matches/quals", variant: "primary" },
      { title: "Quals (Individual)", path: "matches/quals/individual", variant: "info" },
    ]
  },
  {
    title: "Playoffs Match Schedules", paths: [
      { title: "Playoffs", path: "matches/playoffs", variant: "primary" },
      { title: "Playoffs (Individual)", path: "matches/playoffs/individual", variant: "info" },
    ]
  }
]

export const REPORT_BASE_URL = "http://" + window.location.host + REPORTS;

export default function Reports(props: { fta: boolean }) {
  return <Container>
    <h2> Generate Reports </h2>
    <br />

    {
      REPORT_MAP.filter(p => !("fta" in p) || props.fta).map(report => <Row className="mb-4">
        <Col>
          <h3> { report.title } </h3>
          <div className="ml-4">
            {
              report.paths.map(rp => 
                <Button
                  className="mx-1" 
                  href={ REPORT_BASE_URL + "/" + rp.path }
                  variant={rp.variant || "primary"}
                  target="_blank"
                > 
                  { rp.title } 
                </Button>
              )
            }
          </div>
        </Col>
      </Row>)
    }
  </Container>
}