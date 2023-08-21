"use client"
import "./logs.scss";
import React, { useEffect, useState } from "react";
import { withPermission } from "../support/permissions";
import { LogRecord } from "../ws-schema";
import { useWebsocket } from "../support/ws-component";
import { useErrors } from "../support/errors";
import moment from "moment";
import update from "immutability-helper";

export default withPermission(["FTA"], function JMSLogsView() {
  const [ logs, setLogs ] = useState<LogRecord[]>([]);
  const [ lastUpdate, setLastUpdate ] = useState<moment.Moment>(moment());

  const { call } = useWebsocket();
  const { addError } = useErrors();

  const updateAllLogs = async() => {
    let theseLogs: LogRecord[] = [];
    while (true) {
      let data = await call<"logs/get">("logs/get", { since: theseLogs[theseLogs.length - 1]?.timestamp_utc || null });
      if (data.length === 0)
        break;
      for (let d of data)
        theseLogs.push(d);
    }
    setLogs(theseLogs);
  };

  useEffect(() => {
    call<"logs/get">("logs/get", { since: logs[logs.length - 1]?.timestamp_utc || null })
      .then(data => setLogs(update(logs, { $push: data })));
  }, [ lastUpdate ]);

  useEffect(() => {
    updateAllLogs();
    const interval = setInterval(() => setLastUpdate(moment()), 1000);
    return () => clearInterval(interval);
  }, []);

  return <React.Fragment>
    <h3 className="mt-3"> JMS Logs </h3>
    {
      logs.map(log => <p key={log.id} className="log-line" data-level={log.level}>
        <span className="log-time"> { moment.unix(log.timestamp_utc).format("YYYY-MM-DD HH:mm:ss ZZ") } </span>
        <span className="log-level"> { log.level } </span>
        <span className="log-module"> { log.module } </span>
        <span className="log-separator"> &gt; </span>
        <span className="log-message"> { log.message } </span>
        <span className="log-location"> {"[at"} { log.file || "??" }:{ log.line || "??" }{"]"} </span>
      </p>)
    }
  </React.Fragment>
});