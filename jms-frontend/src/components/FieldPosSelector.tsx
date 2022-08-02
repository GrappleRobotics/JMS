import React from "react";
import { Button, Col, Row } from "react-bootstrap";
import { Link } from "react-router-dom";
import { capitalise } from "support/strings";
import { withVal } from "support/util";
import { AllianceStation, AllianceStationId } from "ws-schema";

type PosSelectorProps = {
  title?: React.ReactNode,
  className?: string,
  children?: React.ReactNode[],
  leftChildren?: React.ReactNode | React.ReactNode[],
  rightChildren?: React.ReactNode | React.ReactNode[],
  img?: string
};

export class PosSelector extends React.PureComponent<PosSelectorProps> {
  render() {
    const { title, className, children, img, leftChildren, rightChildren, ...props } = this.props;
    return <Col className={`pos-selector-container ${className || ''}`} { ...props }>
      { withVal(title, t => <Row className="pos-selector-title">
        {
          typeof t === "string" ? <h3 className="text-center mb-4"> {t} </h3> : t
        }
      </Row>) }

      <Row className="pos-selector-row">
        <Col className="pos-selector-left"> { leftChildren } </Col>
        <Col className="pos-selector-image-container">
          { children }
          <img className="pos-selector-image" src={img} />
        </Col>
        <Col className="pos-selector-right"> { rightChildren } </Col>
      </Row>
    </Col>
  }
}


export const FIELD_IMG = "/img/game/field.png";

export default function FieldPosSelector(props: Omit<PosSelectorProps, 'img'>) {
  return <PosSelector img={FIELD_IMG} className={`field-pos-selector ${props.className || ''}`} {...props} />
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