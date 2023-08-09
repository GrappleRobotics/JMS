"use client"
import { withConfirm } from "@/app/components/Confirm";
import MatchSchedule from "@/app/match_schedule";
import { useErrors } from "@/app/support/errors";
import { withPermission } from "@/app/support/permissions";
import { useWebsocket } from "@/app/support/ws-component";
import { Match } from "@/app/ws-schema";
import { faInfoCircle } from "@fortawesome/free-solid-svg-icons";
import { FontAwesomeIcon } from "@fortawesome/react-fontawesome";
import Link from "next/link";
import React from "react";
import { useEffect, useState } from "react";
import { Button } from "react-bootstrap";

export default withPermission(["ManagePlayoffs"], function EventWizardPlayoffs() {
  const [ matches, setMatches ] = useState<Match[]>([]);
  const { call, subscribe, unsubscribe } = useWebsocket();
  const { addError } = useErrors();

  useEffect(() => {
    const cb = [
      subscribe<"matches/matches">("matches/matches", m => setMatches(m.filter(x => x.match_type === "Playoff" || x.match_type === "Final"))),
    ];
    return () => unsubscribe(cb);
  }, []);

  return <React.Fragment>
    <h3> View Playoff Schedule </h3>
    <p className="text-muted"> <FontAwesomeIcon icon={faInfoCircle} /> Playoff generation settings are located in the <Link href="/wizard/event">Event Settings</Link> </p>

    <Button variant="success" onClick={() => call<"matches/update_playoffs">("matches/update_playoffs", null).catch(addError)}>
      Update Playoff Schedule
    </Button> &nbsp;
    <Button variant="danger" onClick={() => withConfirm(() => call<"matches/reset_playoffs">("matches/reset_playoffs", null).catch(addError))}>
      Reset Playoffs
    </Button>
    <br /> <br />
    <MatchSchedule matches={matches} />
  </React.Fragment>
});