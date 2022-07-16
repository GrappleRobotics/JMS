import React from 'react';
import { Button, ButtonProps, Col, Row } from 'react-bootstrap';
import { ArenaMessageAudienceDisplaySet2JMS, ArenaSignal, ArenaState } from 'ws-schema';

class MatchFlowButton extends React.PureComponent<ButtonProps & { arenaState?: ArenaState, targetState: ArenaState["state"] }> {
  render() {
    let { arenaState, targetState, className, ...props } = this.props;

    return <Button
      className={`match-flow-btn ${className || ""}`}
      data-target={targetState}
      active={arenaState?.state === targetState}
      {...props}
    />
  }
}

type MatchFlowProps = {
  onSignal: (signal: ArenaSignal) => void,
  onAudienceDisplay: (display: ArenaMessageAudienceDisplaySet2JMS) => void,
  state?: ArenaState,
  matchLoaded: boolean
};

export default class MatchFlow extends React.PureComponent<MatchFlowProps> {
  render() {
    let { state, onSignal, matchLoaded } = this.props;
    return <Row>
      <Col>
        <Row>
          <Col>
            <MatchFlowButton
              arenaState={state}
              targetState="Prestart"
              onClick={() => onSignal("Prestart")}
              disabled={!(matchLoaded && ((state?.state === "Idle" && state.ready) || (state?.state === "Prestart" && state.ready)))}
            >
              Prestart
            </MatchFlowButton>
          </Col>
          <Col>
            <Button
              className="match-flow-btn"
              data-target="MatchPreview"
              variant="warning"
              disabled={!matchLoaded}
              onClick={() => this.props.onAudienceDisplay("MatchPreview")}
            >
              Match Preview
            </Button>
          </Col>
          <Col>
            <MatchFlowButton 
              arenaState={state} 
              targetState="MatchArmed"
              variant="hazard-yellow"
              onClick={() => this.props.onSignal("MatchArm")}
              disabled={!(state?.state === "Prestart" && state.ready)}
            >
              Arm Match
            </MatchFlowButton>
          </Col>
          <Col>
            <MatchFlowButton
              arenaState={state}
              targetState="MatchPlay"
              variant="hazard-yellow"
              onClick={() => this.props.onSignal("MatchPlay")}
              disabled={state?.state !== "MatchArmed"}
            >
              Match Play
            </MatchFlowButton>
          </Col>
          <Col>
            <MatchFlowButton
              arenaState={state}
              targetState="MatchCommit"
              onClick={() => this.props.onSignal("MatchCommit")}
              disabled={state?.state !== "MatchComplete"}
            >
              Commit Scores
            </MatchFlowButton>
          </Col>
        </Row>
        <br />
        <Row>
          <Col>
            {
              this.props.state?.state === "Estop" ? 
                <MatchFlowButton
                  arenaState={state}
                  targetState="EstopReset"
                  variant="estop-reset"
                  onClick={() => this.props.onSignal("EstopReset")}
                > Reset Emergency Stop </MatchFlowButton> :
                <MatchFlowButton
                  arenaState={state}
                  targetState="Estop"
                  variant="estop"
                  onClick={() => this.props.onSignal("Estop")}
                > EMERGENCY STOP </MatchFlowButton>
            }
          </Col>
        </Row>
      </Col>
    </Row>
  }
}