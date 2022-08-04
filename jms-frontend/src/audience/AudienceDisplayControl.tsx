import BufferedFormControl from "components/elements/BufferedFormControl";
import { confirmModal } from "components/elements/Confirm";
import Paginate from "components/elements/Paginate";
import React from "react";
import { Col, Container, Row, Button, Modal } from "react-bootstrap";
import { nullIfEmpty } from "support/strings";
import { withVal } from "support/util";
import { WebsocketComponent } from "support/ws-component";
import { Award, ArenaMessageAudienceDisplaySet2JMS, SerializedMatch } from "ws-schema";

type SceneT = ArenaMessageAudienceDisplaySet2JMS;

type AudienceDisplayControlState = {
  awards: Award[],
  matches: SerializedMatch[]
};

export default class AudienceDisplayControl extends WebsocketComponent<{}, AudienceDisplayControlState> {
  readonly state: AudienceDisplayControlState = {
    awards: [],
    matches: []
  };

  componentDidMount = () => this.handles = [
    this.listen("Event/Award/CurrentAll", "awards"),
    this.listen("Match/All", "matches")
  ];
  
  change = (scene: SceneT) => {
    this.send({ Arena: { AudienceDisplay: { Set: scene } } })
  }

  customMessage = () => {
    confirmModal("", {
      data: "",
      title: "Show Custom Message",
      okText: "Display",
      renderInner: (msg: string, onUpdate) => <React.Fragment>
        <p> Custom Message: </p>
        <BufferedFormControl
          auto
          autofocus
          type="text"
          value={msg}
          onUpdate={(v) => onUpdate(String(v))}
        />
      </React.Fragment>
    }).then(msg => {
      if (nullIfEmpty(msg) != null) {
        this.change({ CustomMessage: msg })
      }
    })
  }

  matchResults = () => {
    confirmModal("", {
      data: "",
      title: "Show Match Results",
      render: (ok, cancel) => <React.Fragment>
        <Modal.Body>
          <Paginate itemsPerPage={10}>
            {
              this.state.matches.filter(m => m.played).reverse().map(m => (
                <Button className="btn-block m-1" onClick={() => ok(m.id || "")}> { m.name } </Button>
              ))
            }
          </Paginate>
        </Modal.Body>
        <Modal.Footer>
          <Button onClick={cancel} variant="secondary"> Cancel </Button>
        </Modal.Footer>
      </React.Fragment>
    }).then(data => {
      withVal(nullIfEmpty(data), id => this.change({ MatchResults: id }))
    })
  }

  render() {
    const sections: { title: string, scenes: { id: string, name: string, scene: SceneT | (() => void) }[] }[] = [
      {
        title: "General Purpose",
        scenes: [
          { id: "field", name: "Field", scene: "Field" },
          { id: "custom", name: "Custom...", scene: this.customMessage }
        ]
      },
      {
        title: "Match Control",
        scenes: [
          { id: "match_preview", name: "Match Preview", scene: "MatchPreview" },
          { id: "match_play", name: "Match Play", scene: "MatchPlay" },
          { id: "match_results_choose", name: "Match Results (choose)", scene: this.matchResults },
          { id: "match_results", name: "Match Results (latest)", scene: { MatchResults: null } },
        ]
      },
      {
        title: "Alliance Selections",
        scenes: [
          { id: "alliance_selection", name: "Alliance Selections", scene: "AllianceSelection" }
        ]
      }
    ];

    return <Container className="audience-control">
      <h2> Audience Display Control </h2>
      <p> If displays are not yet ready to display data (e.g. match is not loaded), displays will default to a blank 
        field view until data is ready. </p>
      <br />

      {
        sections.map(section => <Row className="mb-4">
          <Col>
            <h3> { section.title } </h3>
            <div className="ml-4">
              {
                section.scenes.map(scene => <React.Fragment>
                  <Button
                    className="btn-block m-1"
                    size="lg"
                    onClick={() => typeof scene.scene === "function" ? scene.scene() : this.change(scene.scene)}
                    data-scene={scene.id}
                  >
                    { scene.name }
                  </Button> <br />
                </React.Fragment>)
              }
            </div>
          </Col>
        </Row>)
      }

      <Row className="mb-4">
        <Col>
          <h3> Awards </h3>
          <div className="ml-4">
            {
              this.state.awards.map(award => <React.Fragment>
                <Button
                  className="m-1 px-5 award-btn"
                  onClick={() => this.change({ Award: award.id! })}
                  disabled={award.recipients.length === 0}
                >
                  { award.name }
                </Button>
              </React.Fragment>)
            }
          </div>
        </Col>
      </Row>

    </Container>
  }
}