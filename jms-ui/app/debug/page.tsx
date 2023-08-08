"use client";

import React from "react";
import { withPermission } from "../support/permissions";
import { FontAwesomeIcon } from "@fortawesome/react-fontawesome";
import { faInfoCircle } from "@fortawesome/free-solid-svg-icons";
import { useWebsocket } from "../support/ws-component";
import { useErrors } from "../support/errors";
import { Button } from "react-bootstrap";

export default withPermission(["FTA"], function DebugPage() {
  const { call } = useWebsocket();
  const { addError } = useErrors();
  
  return <div className="mt-3">
    <h3>Debug Controls</h3>
    <p className="text-muted"> <FontAwesomeIcon icon={faInfoCircle} /> &nbsp; These tools are provided for debugging purposes only. Don't use these unless you're certain you know what they do! </p>

    <h4> Trigger Events </h4>
    <Button variant="success" onClick={() => call<"scoring/update_rankings">("scoring/update_rankings", null).catch(addError)}>
      Recalculate Team Rankings
    </Button>
  </div>
});