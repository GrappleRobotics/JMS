import React from "react";
import { Button, Col, Row } from "react-bootstrap";
import { Link } from "react-router-dom";
import { capitalise } from "support/strings";
import { withVal } from "support/util";
import { AllianceStation, AllianceStationId } from "ws-schema";

type FieldPosSelectorProps = {
  title?: React.ReactNode,
  className?: string,
  children?: React.ReactNode[]
};

export default class FieldPosSelector extends React.PureComponent<FieldPosSelectorProps> {
  render() {
    const { title, className, children } = this.props;
    return <Col className={`field-pos-selector-container ${className || ''}`}>
      { withVal(title, t => <Row className="field-pos-selector-title">
        {
          typeof t === "string" ? <h3 className="text-center mb-4"> {t} </h3> : t
        }
      </Row>) }

      <Row className="field-pos-selector-row">
        <Col className="field-pos-selector-image-container">
          { children }
          <img className="field-pos-selector-image" src="/img/game/field.png" />
        </Col>
      </Row>
    </Col>
  }
}

type FieldStationSelectorProps = {
  title: React.ReactNode,
  className?: string,
  stations: AllianceStation[],
  onSelect?: (v: AllianceStation) => void,
  children?: React.ReactNode[]
};

export class TeamSelector extends React.PureComponent<FieldStationSelectorProps> {
  static defaultProps = {
    title: "Select Alliance Station"
  };

  render() {
    const { title, className, stations, onSelect, children } = this.props;

    return <FieldPosSelector className={`team-selector ${className || ""}`} title={title}>
      {
        stations.map((s, i) => {
          const alliance = s.station.alliance;
          const position = s.station.station;
          const team = s.team;
          
          const btnContent = <React.Fragment>
            { capitalise(alliance) } { position } { team }
          </React.Fragment>

          if (onSelect) {
            return <Button key={i} data-alliance={alliance} data-position={position} onClick={() => onSelect(s)}>
              { btnContent }
            </Button>
          } else {
            return <Link key={i} to={`${alliance}-${position}`}>
              <Button data-alliance={alliance} data-position={position}>
                { btnContent }
              </Button>
            </Link>
          }
        })
      }
      
      { children }
    </FieldPosSelector>
  }
}