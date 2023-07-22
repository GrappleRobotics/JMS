import confirmBool from 'components/elements/Confirm';
import { ResourceRequirementMinimap } from 'components/ResourceComponents';
import React from 'react';
import { Button, ButtonProps, Col, Row } from 'react-bootstrap';
import { SerialisedAllianceStation, ArenaMessageAudienceDisplaySet2JMS, ArenaSignal, ArenaState, ResourceRequirementStatus } from 'ws-schema';

class MatchFlowButton extends React.PureComponent<ButtonProps & { arenaState?: ArenaState, targetState?: ArenaState["state"] }> {
  render() {
    let { arenaState, targetState, className, ...props } = this.props;

    return <Button
      className={`match-flow-btn ${className || ""}`}
      data-target={targetState}
      active={targetState != undefined && arenaState?.state === targetState}
      {...props}
    />
  }
}

type MatchFlowProps = {
  onSignal: (signal: ArenaSignal) => void,
  onAudienceDisplay: (display: ArenaMessageAudienceDisplaySet2JMS) => void,
  state?: ArenaState,
  matchLoaded: boolean,
  bigEstop?: boolean,
  resources?: ResourceRequirementStatus,
  stations: SerialisedAllianceStation[],
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
    ).then(b => b ? this.props.onSignal({ MatchArm: { force: true } }) : {});
  }

  render() {
    let { state, onSignal, matchLoaded, resources, stations, bigEstop } = this.props;
    const teamsOk = stations.every(s => s.can_arm);
    const resourceOk = resources ? resources.ready : true;

    if (state?.state === "Estop") {
      return <Row>
        <Col>
          <MatchFlowButton
            arenaState={state}
            variant="estop-reset"
            onClick={() => this.props.onSignal("EstopReset")}
          > Reset Emergency Stop </MatchFlowButton>
        </Col>
      </Row>
    }

    return <Row>
      <Col>
        <Row>
          <Col>
            <MatchFlowButton
              arenaState={state}
              targetState="Prestart"
              onClick={() => onSignal(state?.state == "Prestart" ? "PrestartUndo" : "Prestart")}
              disabled={!(matchLoaded && ((state?.state === "Idle" && state.net_ready) || (state?.state === "Prestart" && state.net_ready)))}
            >
              { state?.state == "Prestart" ? "Revert Prestart" : "Prestart" }
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
                resourceOk ? this.props.onSignal({ MatchArm: { force: false } }) : this.forceArm(resources!)
              }}
              disabled={!(state?.state === "Prestart" && state.net_ready && teamsOk)}
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
              onClick={() => this.props.onSignal("MatchCommit")}
              disabled={state?.state !== "MatchComplete"}
            >
              Commit Scores
            </MatchFlowButton>
          </Col>
        </Row>
        {
          bigEstop && <React.Fragment>
            <br />
            <Row>
              <Col>
                <MatchFlowButton
                  arenaState={state}
                  variant="estop"
                  onClick={() => this.props.onSignal("Estop")}
                > EMERGENCY STOP </MatchFlowButton>
              </Col>
            </Row>
          </React.Fragment>
        }
      </Col>
    </Row>
  }
}