import React from "react";
import { Button, Col, Row } from "react-bootstrap";
import { withVal, withValU } from "support/util";
import { role2id } from "support/ws-additional";
import { ResourceRole } from "ws-schema";
import { ResourceRoleLabel } from "./ResourceComponents";

type PosSelectorProps = {
  title?: React.ReactNode,
  className?: string,
  children?: React.ReactNode | React.ReactNode[],
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
        {
          withValU(leftChildren, left => <Col md={2} className="pos-selector-left"> { left } </Col>)
        }
        <Col>
          <div className="pos-selector-image-container">
            { children }
            <img className="pos-selector-image" src={img} />
          </div>
        </Col>
        {
          withValU(rightChildren, right => <Col md={2} className="pos-selector-right"> { right } </Col>)
        }
      </Row>
    </Col>
  }
}

export function FieldResource(props: { role: ResourceRole, fta?: boolean, children?: React.ReactNode | React.ReactNode[] }) {
  return <div className="field-pos-resource" data-role={role2id(props.role)} data-fta={props.fta}>
    { props.children }
  </div>
}

export const FIELD_IMG = "/img/game/field.png";

export type FieldPosSelectorProps = Omit<PosSelectorProps, 'img'>;

export default function FieldPosSelector(props: Omit<PosSelectorProps, 'img'>) {
  return <PosSelector img={FIELD_IMG} className={`field-pos-selector ${props.className || ''}`} {...props} />
}

export type FieldResourceSelectorProps<T extends ResourceRole> = {
  options: T[],
  labels?: React.ReactNode[],
  wrap?: (role: T, child: React.ReactNode) => React.ReactNode,
  onSelect?: (role: T) => void
} & FieldPosSelectorProps;

export function FieldResourceSelector<T extends ResourceRole>(props: FieldResourceSelectorProps<T>) {
  const { options, onSelect, labels, wrap, ...p } = props;

  return <FieldPosSelector { ...p }>
    {
      options.map((r, i) => (
        <FieldResource key={i} role={r}>
          {
            (wrap || ((r: T, child: React.ReactNode) => child))(r, <Button onClick={() => onSelect ? onSelect(r) : {}}>
              {
                labels ? labels[i] : <ResourceRoleLabel role={r} />
              }
            </Button>)
          }
        </FieldResource>
      ))
    }
  </FieldPosSelector>
}
