import React from 'react';
import { Row, Col, Button } from 'react-bootstrap';

class MatchFlowButton extends React.PureComponent {
  render() {
    let { stateName, ...props } = this.props;
    let active = this.props.state?.state === stateName;
    return <Button 
      size="lg" 
      className="p-3" 
      block
      active={active}
      {...props}
      variant={active ? "primary" : this.props.disabled ? 'secondary' : (this.props.variant || 'info')}
    />
  }
}

export default class MatchFlow extends React.Component {
  render() {
    return <Row>
      <Col>
        <Row>
          <Col>
            <MatchFlowButton 
              state={this.props.state}
              stateName="Prestart"
              onClick={() => this.props.onSignal({ signal: "Prestart" })}
              disabled={!( this.props.match != null && (this.props.state?.state === "Idle" || this.props.state?.state === "Prestart") )}
            >
              Prestart
            </MatchFlowButton>
          </Col>
          <Col>
            <Button
              size="lg"
              className="p-3"
              block
              variant={!!!this.props.match ? "secondary" : "warning"}
              disabled={!!!this.props.match}
              onClick={() => this.props.onAudienceDisplay("MatchPreview", null)}
            >
              Match Preview
            </Button>
          </Col>
          <Col>
            <MatchFlowButton 
              state={this.props.state} 
              stateName="MatchArmed"
              variant="hazard-dark"
              onClick={() => this.props.onSignal({ signal: "MatchArm" })}
              disabled={!( this.props.state?.state === "Prestart" && this.props.state?.ready )}
            >
              Arm Match
            </MatchFlowButton>
          </Col>
          <Col>
            <MatchFlowButton
              state={this.props.state}
              stateName="MatchPlay"
              variant="hazard-dark"
              onClick={() => this.props.onSignal({ signal: "MatchPlay" })}
              disabled={!( this.props.state?.state === "MatchArmed" )}
            >
              Match Play
            </MatchFlowButton>
          </Col>
          <Col>
            <MatchFlowButton
              state={this.props.state}
              stateName="MatchCommit"
              onClick={() => this.props.onSignal({ signal: "MatchCommit" })}
              disabled={!( this.props.state?.state === "MatchComplete" )}
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
                  state={this.props.state}
                  stateName="EstopReset"
                  variant="warning"
                  onClick={() => this.props.onSignal({ signal: "EstopReset" })}
                > Reset Emergency Stop </MatchFlowButton> :
                <MatchFlowButton
                  state={this.props.state}
                  stateName="Estop"
                  variant="hazard-red-dark"
                  onClick={() => this.props.onSignal({ signal: "Estop" })}
                > EMERGENCY STOP </MatchFlowButton>
            }
          </Col>
        </Row>
      </Col>
    </Row>
  }
}