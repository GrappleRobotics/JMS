"use client"

import BufferedFormControl from "@/app/components/BufferedFormControl";
import { useErrors } from "@/app/support/errors";
import { withPermission } from "@/app/support/permissions"
import { nullIfEmpty } from "@/app/support/strings";
import { useWebsocket } from "@/app/support/ws-component";
import { EventDetails } from "@/app/ws-schema";
import React, { useEffect, useState } from "react";
import { Card, Col, Form, InputGroup, Row } from "react-bootstrap";
import update, { Spec } from "immutability-helper";
import { SketchPicker } from 'react-color';

export default withPermission("Admin", function EventWizardUsers() {
  const [ details, setDetails ] = useState<EventDetails | null>(null);
  const { call, subscribe } = useWebsocket();
  const { addError } = useErrors();

  useEffect(() => {
    subscribe<"event/details">("event/details", setDetails);
  }, []);

  const updateDetails = (spec: Spec<EventDetails>) => {
    call<"event/update">("event/update", { details: update(details!, spec) })
      .then(setDetails)
      .catch(addError);
  }

  return <React.Fragment>
    <h3> Event Details </h3>
    <br />

    <Row>
      <Col>
        <InputGroup>
          <InputGroup.Text>Event Code</InputGroup.Text>
          <BufferedFormControl
            type="text"
            placeholder="2023myevent"
            value={details?.code || ""}
            onUpdate={v => updateDetails({ code: { $set: nullIfEmpty(String(v)) } })}
          />
        </InputGroup>

        <InputGroup className="mt-2">
          <InputGroup.Text>Event Name</InputGroup.Text>
          <BufferedFormControl
            type="text"
            placeholder="My Really Awesome Event"
            value={details?.event_name || ""}
            onUpdate={v => updateDetails({ event_name: { $set: nullIfEmpty(String(v)) } })}
          />
        </InputGroup>
      </Col>
    </Row>

    {/* TODO: Webcast Links for TBA */}

    <Row className="mt-3">
      <Col md="auto" className="mx-2">
        <h6> Event Colour </h6>
        <SketchPicker
          disableAlpha
          presetColors={["#e9ab01", "#1f5fb9", "#901fb9"]}
          color={ details?.av_event_colour }
          onChangeComplete={ c => updateDetails({ av_event_colour: { $set: c.hex } }) }
        />
      </Col>
      <Col md="auto" className="mx-2">
        <h6> Chroma Key Colour </h6>
        <SketchPicker
          disableAlpha
          presetColors={["#000", "#f0f", "#0f0", "#333"]}
          color={ details?.av_chroma_key }
          onChangeComplete={ c => updateDetails({ av_chroma_key: { $set: c.hex } }) }
        />
      </Col>
    </Row>
  </React.Fragment>
});