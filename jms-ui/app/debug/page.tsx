"use client";

import React from "react";
import { withPermission } from "../support/permissions";
import { FontAwesomeIcon } from "@fortawesome/react-fontawesome";
import { faInfoCircle } from "@fortawesome/free-solid-svg-icons";
import { useWebsocket } from "../support/ws-component";
import { useErrors } from "../support/errors";
import { Button } from "react-bootstrap";
import { withConfirm } from "../components/Confirm";

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

    <br /> <br />
    <h4> Matches </h4>
    <Button variant="danger" onClick={() => withConfirm(() => call<"scoring/debug_random_fill">("scoring/debug_random_fill", { ty: "Qualification" }).catch(addError))}>
      Random Fill Matches (Quals)
    </Button> &nbsp;
    <Button variant="danger" onClick={() => withConfirm(() => call<"scoring/debug_random_fill">("scoring/debug_random_fill", { ty: "Playoff" }).catch(addError))}>
      Random Fill Matches (Playoffs)
    </Button> &nbsp;
    <Button variant="danger" onClick={() => withConfirm(() => call<"matches/debug_delete_all">("matches/debug_delete_all", null).catch(addError))}>
      DELETE ALL
    </Button>
  </div>
});