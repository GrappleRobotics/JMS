"use client"
import { Button } from "react-bootstrap";
import UserPage from "../userpage";
import { useWebsocket } from "../support/ws-component";
import { useErrors } from "../support/errors";
import { MatchType, ReportData, WebsocketRpcRequest } from "../ws-schema";
import React, { useState } from "react";
import { PermissionGate } from "../support/permissions";

const MATCH_TYPE_VARIANT: { [k in MatchType]: string } = {
  Test: "danger",
  Qualification: "primary",
  Playoff: "warning",
  Final: "success",
}

export default function Reports() {
  const [loading, setLoading] = useState<boolean>(false);

  const { call } = useWebsocket();
  const { addError } = useErrors();

  const open = (report: ReportData) => {
    console.log("Report: ", report)
    let file = new Blob([new Uint8Array(report.data).buffer], { type: report.mime });
    let file_url = URL.createObjectURL(file);
    window.open(file_url);
  }

  const report = <Path extends WebsocketRpcRequest["path"]>
         (path: Path, args: Extract<WebsocketRpcRequest, { path: Path }>["data"]) =>
  {
    setLoading(true);
    call<Path>(path, args as any).then(data => {
      open(data as any);
      setLoading(false);
    }).catch(addError)
  }

  return <UserPage container>
    <h3 className="mt-3"> Generate Reports </h3>

    <h4 className="mt-2"> Teams </h4>
    <Button disabled={loading} size="lg" onClick={() => report<"reports/teams">("reports/teams", null)}>
      Team Report
    </Button>

    <h4 className="mt-2"> Awards </h4>
    <Button disabled={loading} size="lg" onClick={() => report<"reports/awards">("reports/awards", null)}>
      Award Report
    </Button>

    <h4 className="mt-2"> Rankings </h4>
    <Button disabled={loading} size="lg" onClick={() => report<"reports/rankings">("reports/rankings", null)}>
      Rankings Report
    </Button>

    <h4 className="mt-2"> Matches </h4>
    {
      (["Qualification", "Playoff", "Final"] as MatchType[]).map(match_type => <React.Fragment key={match_type as string}>
        <Button disabled={loading} className="mb-2" size="lg" variant={MATCH_TYPE_VARIANT[match_type]} onClick={() => report<"reports/matches">("reports/matches", { individual: false, match_type })}>
          { match_type }
        </Button> &nbsp;
        <Button disabled={loading} className="mb-2" size="lg" variant={MATCH_TYPE_VARIANT[match_type]} onClick={() => report<"reports/matches">("reports/matches", { individual: true, match_type })}>
          { match_type } (Individual)
        </Button> <br />
      </React.Fragment>)
    }

    <PermissionGate permissions={["FTA"]}>
      <h4>WPA Keys</h4>
      <Button disabled={loading} size="lg" variant="danger" onClick={() => report<"reports/wpa_key">("reports/wpa_key", { csv: false })}>
        WPA Keys (pdf)
      </Button> &nbsp;
      <Button disabled={loading} size="lg" variant="danger" onClick={() => report<"reports/wpa_key">("reports/wpa_key", { csv: true })}>
        WPA Keys (csv)
      </Button>
    </PermissionGate>
  </UserPage>
}