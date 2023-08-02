"use client";
import { useWebsocket } from "./support/ws-component";
import React, { useEffect, useLayoutEffect, useRef, useState } from "react";
import { FontAwesomeIcon } from "@fortawesome/react-fontawesome";
import { faRightFromBracket } from "@fortawesome/free-solid-svg-icons";
import { Alert, Button, Col, Container, Nav, Navbar, Row } from "react-bootstrap";
import "./userpage.scss";
import Link from "next/link";
import { useErrors } from "./support/errors";
import { ArenaState, JmsComponent, Match, SerialisedLoadedMatch } from "./ws-schema";
import { PermissionGate } from "./support/permissions";
import confirmBool from "./components/Confirm";
import JmsWebsocket from "./support/ws";
import moment from "moment";
import "moment-duration-format";
import SimpleTooltip from "./components/SimpleTooltip";

interface UserPageProps {
  container?: boolean,
  space?: boolean,
  children: React.ReactNode
};

export default function UserPage(props: UserPageProps) {
  const { error, removeError } = useErrors();
  const topRef = useRef<HTMLElement>(null);
  const bottomRef = useRef<HTMLElement>(null);

  const [ viewportPos, setViewportPos ] = useState({ top_height: 0, bottom_top: 0 });

  const alert = error !== null && <Alert className="m-3" variant="danger" dismissible onClose={() => removeError()}>{ error }</Alert>

  useEffect(() => {
    const ob = new ResizeObserver(() => {
      setViewportPos({
        top_height: topRef?.current?.clientHeight || 0,
        bottom_top: bottomRef?.current?.getBoundingClientRect()?.y || 0
      })
    });
    ob.observe(topRef?.current!);
    ob.observe(bottomRef?.current!);
    return () => ob.disconnect();
  })

  return <React.Fragment>
    <TopNavbar ref={topRef} />
    <div className="viewport" style={{
      position: 'fixed',
      top: viewportPos.top_height,
      height: viewportPos.bottom_top - viewportPos.top_height,
      width: '100vw',
      overflowY: "auto"
    }}>
      {
        props.container ? 
          <Container className={props.space ? "mt-3" : ""}> { alert } { props.children } </Container> 
          : <React.Fragment> { alert } { props.children } </React.Fragment>
      }
    </div>
    <BottomNavbar ref={bottomRef} />
  </React.Fragment>
};

const ARENA_STATE_MAP: { [k in ArenaState["state"]]: string } = {
  "Init": "Initialising...",
  "Idle": "Idle",
  "Estop": "EMERGENCY STOP",
  "MatchArmed": "ARMED",
  "MatchComplete": "Match Complete",
  "MatchPlay": "Match Play",
  "Prestart": "Prestart",
  "Reset": "Resetting..."
};

const TopNavbar = React.forwardRef<HTMLElement>(function TopNavbar(props: {}, ref) {
  const { user, connected, logout, subscribe, unsubscribe, call } = useWebsocket();
  const { addError } = useErrors();

  const [ arenaState, setArenaState ] = useState<ArenaState>();
  const [ currentMatch, setCurrentMatch ] = useState<SerialisedLoadedMatch | null>(null);

  useEffect(() => {
    let handles = [
      subscribe<"arena/state">("arena/state", setArenaState),
      subscribe<"arena/current_match">("arena/current_match", setCurrentMatch),
    ];
    return () => unsubscribe(handles);
  }, []);

  let state_comment = arenaState ? ARENA_STATE_MAP[arenaState.state] : "Waiting...";
  if (currentMatch) {
    if (currentMatch.state in ["Auto", "Pause", "Teleop"]) 
      state_comment = `${currentMatch.state} (T-${currentMatch.remaining.toFixed(0)})`;
  }

  return <Navbar ref={ref} className="navbar-top" data-connected={connected} data-arena-state={arenaState?.state} data-match-state={currentMatch?.state} variant="dark">
    <Container>
      <PermissionGate permissions={["Estop"]}>
        { arenaState?.state === "Estop" ? <PermissionGate permissions={["FTA"]}>
          <Button className="mx-3" variant="estop-reset" onClick={() => call<"arena/signal">("arena/signal", { signal: "EstopReset" })}>RESET</Button>
        </PermissionGate> : <Button className="mx-3" variant="estop" onClick={() => estopModal(call, addError)}>E-STOP</Button>}
      </PermissionGate>

      <Navbar.Brand>
        <Link href="/" style={{ textDecoration: "none", color: "white" }}> <strong>JMS</strong> </Link>
        &nbsp; &nbsp;
        <strong style={{ color: "white" }}>{ connected ? state_comment : "[DISCONNECTED]" }</strong>
      </Navbar.Brand>
      <Navbar.Toggle/>
      <Navbar.Collapse className="justify-content-end">
        <Nav>
          {
            user ? <Navbar.Text>
              Hello <span style={{ color: "white" }}>{ user.username }</span> &nbsp;
              <Button size="sm" variant="red" onClick={() => logout()}> <FontAwesomeIcon icon={faRightFromBracket} /> </Button>
            </Navbar.Text> : <Link href="/login"> <Button variant="secondary"> LOGIN </Button> </Link>
          }
        </Nav>
      </Navbar.Collapse>
    </Container>
  </Navbar>;
});

const BottomNavbar = React.forwardRef<HTMLElement>(function(props: {}, ref) {
  const { subscribe, unsubscribe, call } = useWebsocket();
  const [ components, setComponents ] = useState<[string, JmsComponent[]]>();
  const [ nextMatch, setNextMatch ] = useState<Match | null>(null);
  const [ now, setNow ] = useState<moment.Moment>(moment());

  useEffect(() => {
    let cbs = [
      subscribe<"components/components">("components/components", setComponents),
      subscribe<"matches/next">("matches/next", setNextMatch),
    ];
    return () => unsubscribe(cbs);
  }, []);

  const time_diff = nextMatch && moment(nextMatch.start_time).diff(now);
  let rendered_time = <React.Fragment />;
  // @ts-ignore
  let format = (d: moment.Duration) => d.format("d[d] h[h] m[m]", { trim: "both" });

  if (time_diff !== null && time_diff >= 0) {
    // @ts-ignore
    // rendered_time = `${moment.duration(time_diff).format("d[d] h[h] m[m]", { trim: "both" })} AHEAD`;
    rendered_time = <strong className="text-good">
      { format(moment.duration(time_diff)) } AHEAD
    </strong>
  } else if (time_diff !== null) {
    rendered_time = <strong className="text-bad">
      { format(moment.duration(-time_diff)) } BEHIND
    </strong>
  }

  return <Row ref={ref} className="navbar-bottom">
    <Col>
      {
        components?.[1]?.sort((a, b) => a.symbol.localeCompare(b.symbol))?.map(c => <ComponentIndicator key={c.id} time={moment(components[0])} component={c} />)
      }
    </Col>
    <Col md="auto">
      { rendered_time }
    </Col>
    <Col>
    </Col>
  </Row>
})

async function estopModal(call: JmsWebsocket["call"], addError: (e: string) => void) {
  const subtitle = <p className="estop-subtitle text-muted">
    Anyone can E-Stop the match. <br />
    E-Stop if there is a safety threat or as instructed by the FTA. <br />
    <strong className="text-danger"> Robot Faults are NOT Field E-Stop conditions. </strong>
  </p>

  let result = await confirmBool(subtitle, {
    size: "xl",
    okBtn: {
      size: "lg",
      className: "estop-big",
      variant: "estop",
      children: "EMERGENCY STOP"
    },
    cancelBtn: {
      size: "lg",
      className: "btn-block",
      children: "CANCEL",
      variant: "secondary"
    }
  });

  if (result) {
    call<"arena/signal">("arena/signal", { signal: "Estop" }).catch(addError);
  }
}

function ComponentIndicator({ time, component }: { time: moment.Moment, component: JmsComponent }) {
  return <SimpleTooltip id={ component.id } tip={component.name}>
    <div className="jms-component" data-heartbeat={time.diff(moment(component.last_tick)) <= component.timeout_ms}>
      { component.symbol }
    </div>
  </SimpleTooltip>
}