import confirmBool from 'components/elements/Confirm';
import { ResourceRequirementMinimap } from 'components/ResourceComponents';
import React from 'react';
import { Button, ButtonProps, Col, Row } from 'react-bootstrap';
import { SerialisedAllianceStation, ArenaMessageAudienceDisplaySet2JMS, ArenaSignal, ArenaState, ResourceRequirementStatus } from 'ws-schema';

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
  matchLoaded: boolean,
  resources?: ResourceRequirementStatus,
  stations: SerialisedAllianceStation[]
};

export default class MatchFlow extends React.PureComponent<MatchFlowProps> {
  forceArm = (resources: ResourceRequirementStatus) => {
    confirmBool(<div className="text-center">
        <h3 className="text-danger"> <strong>RESOURCES ARE NOT READY</strong> </h3>
        <p> Are you sure you want to <strong className="text-danger"> ARM MATCH? </strong> </p>
        <br />
        <ResourceRequirementMinimap status={resources} />
        <br />
      </div>,
      { title: undefined, okText: "Ignore Resource Requirements", okVariant: "hazard-red", size: "lg" }
    ).then(b => b ? this.props.onSignal({ MatchArm: true }) : {});
  }

  render() {
    let { state, onSignal, matchLoaded, resources, stations } = this.props;
    const teamsOk = stations.every(s => s.can_arm);
    const resourceOk = resources ? resources.ready : true;

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
              variant={resourceOk ? "hazard-yellow" : "hazard-red"}
              onClick={() => {
                resourceOk ? this.props.onSignal({ MatchArm: false }) : this.forceArm(resources!)
              }}
              disabled={!(state?.state === "Prestart" && state.ready && teamsOk)}
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
        {
          this.props.state?.state === "Prestart" && this.props.state?.ready ?
            <Row>
              <Col>
                <MatchFlowButton
                  arenaState={state}
                  targetState="Idle"
                  onClick={() => this.props.onSignal("Idle")}
                  variant="secondary"
                >
                  REVERT PRESTART
                </MatchFlowButton>
              </Col>
            </Row> : <React.Fragment />
        }
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