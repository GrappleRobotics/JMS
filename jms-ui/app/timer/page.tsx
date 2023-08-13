"use client";
import "./timer.scss";
import { useEffect, useState } from "react";
import { SerialisedLoadedMatch } from "../ws-schema";
import { useWebsocket } from "../support/ws-component";

export default function MatchTimer() {
  const [ currentMatch, setCurrentMatch ] = useState<SerialisedLoadedMatch | null>(null);
  const { subscribe, unsubscribe } = useWebsocket();

  useEffect(() => {
    const cbs = [ subscribe<"arena/current_match">("arena/current_match", setCurrentMatch) ];
    return () => unsubscribe(cbs);
  }, []);

  return <div className="timer">
    <div> { currentMatch?.remaining ? Math.floor(currentMatch.remaining / 1000) : "----" } </div>
  </div>
}