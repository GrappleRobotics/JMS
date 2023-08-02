"use client"
import "./schedule.scss";
import { withPermission } from "@/app/support/permissions";
import FullCalendar from "@fullcalendar/react";
import interactionPlugin from "@fullcalendar/interaction";
import timeGridPlugin from "@fullcalendar/timegrid";
import momentPlugin from "@fullcalendar/moment";
import bootstrap5Plugin from "@fullcalendar/bootstrap5";
import React, { useEffect, useState } from "react";
import moment from "moment";
import { ScheduleBlock, ScheduleBlockType, ScheduleBlockUpdate } from "@/app/ws-schema";
import { useWebsocket } from "@/app/support/ws-component";
import { useErrors } from "@/app/support/errors";
import JmsWebsocket from "@/app/support/ws";
import { DateSelectArg, EventApi, EventInput } from "@fullcalendar/core/index.js";
import { confirmModal } from "@/app/components/Confirm";
import { Form, InputGroup } from "react-bootstrap";
import BufferedFormControl from "@/app/components/BufferedFormControl";
import update from "immutability-helper";

const ELEMENT_FORMAT = "YYYY-MM-DD[T]HH:mm";
const TYPE_MAP: { [k in ScheduleBlockType["type"]]: string } = {
  General: "General Purpose",
  Ceremonies: "Ceremonies",
  Lunch: "Lunch",
  FieldTests: "Field Test",
  SetupTeardown: "Setup / Teardown",
  Qualification: "Qualifications",
  Playoff: "Playoffs"
};

const TYPE_DEFAULTS: { [k in ScheduleBlockType["type"]]: ScheduleBlockType } = {
  General: { type: "General" },
  Ceremonies: { type: "Ceremonies" },
  Lunch: { type: "Lunch" },
  FieldTests: { type: "FieldTests" },
  SetupTeardown: { type: "SetupTeardown" },
  Qualification: { type: "Qualification", cycle_time: 11*60*1000 },
  Playoff: { type: "Playoff" }
};

const TYPE_COLOUR_MAP: { [k in ScheduleBlockType["type"]]: string } = {
  General: "#2196F3",
  Ceremonies: "#9C27B0",
  Lunch: "#616161",
  FieldTests: "#2E7D32",
  SetupTeardown: "#616161",
  Qualification: "#3F51B5",
  Playoff: "#EF6C00"
};

export default withPermission(["ManageSchedule"], function EventWizardSchedule() {
  const [ blocks, setBlocks ] = useState<ScheduleBlock[]>([]);
  const [ nTeams, setNTeams ] = useState<number>(0);

  const { call, subscribe, unsubscribe } = useWebsocket();
  const { addError } = useErrors();

  const refresh = () => {
    call<"event/schedule_get">("event/schedule_get", null).then(setBlocks).catch(addError);
  }

  useEffect(() => {
    refresh();
    let cbs = [
      subscribe<"team/teams">("team/teams", ts => setNTeams(ts.filter(t => t.schedule).length))
    ];
    return () => unsubscribe(cbs);
  }, [])

  const nMatches = blocks.map(b => {
    if (b.block_type.type === "Qualification")
      return Math.floor( moment(b.end_time).diff(moment(b.start_time)) / b.block_type.cycle_time );
    else
      return 0;
  }).reduce((last, current) => last + current, 0);

  let matches_per_team = Math.floor((nMatches * 6) / (nTeams || 1));

  return <React.Fragment>
    <h3>Manage Event Schedule</h3>
    <span>
      <i className="text-muted">Quals</i> &nbsp;
      <strong>{ nMatches }</strong> matches
      <span className="text-muted mx-2">•</span>
      <strong>{ nTeams >= 6 ? matches_per_team : "--" }</strong> per team
      <span className="text-muted">
        &nbsp; <i>+ {
          nTeams >= 6 ? ( (nMatches * 6) - (matches_per_team * nTeams)) : "--"
        } </i>
      </span>
    </span>
    <hr />

    <FullCalendar
      plugins={[ momentPlugin, timeGridPlugin, bootstrap5Plugin, interactionPlugin ]}
      editable={true}
      selectable={true}
      selectMirror={true}
      initialView="timeGrid3Day"
      titleFormat="MMMM D, YYYY"
      dateIncrement={ { days: 1 } }
      snapDuration={ { minutes: 15 } }
      headerToolbar={{
        left: 'prev,next today',
        center: 'title',
        right: 'timeGrid3Day,timeGrid5Day'
      }}
      views={{
        timeGrid3Day: { type: 'timeGrid', duration: { days: 3 }, allDaySlot: false, buttonText: "3 Day" },
        timeGrid5Day: { type: 'timeGrid', duration: { days: 5 }, allDaySlot: false, buttonText: "5 Day" },
      }}
      events = {blocks.map(block => { return {
        id: block.id,
        title: renderTitle(block),
        start: block.start_time, end: block.end_time,
        color: TYPE_COLOUR_MAP[block.block_type.type],
        extendedProps: {
          type: block.block_type,
          name: block.name
        }
      }})}
      select={(e) => newEventModal(e, call, refresh, addError)}
      eventClick={(e) => editEventModal(e.event, call, refresh, addError)}
      eventChange={(e) => {
        call<"event/schedule_edit">("event/schedule_edit", { block_id: e.event.id, updates: [
          { start_time: e.event.start!.toISOString() }, { end_time: e.event.end!.toISOString() }
        ]}).then(refresh).catch(addError)
      }}
    />
  </React.Fragment>
});

function renderTitle(block: ScheduleBlock) {
  if (block.block_type.type === "Qualification") {
    const n_matches = Math.floor( moment(block.end_time).diff(moment(block.start_time)) / block.block_type.cycle_time );
    return `${block.name} (${block.block_type.type} - ${n_matches} matches)`
  } else {
    return `${block.name} (${block.block_type.type})`
  }
}

async function editEventModal(event: EventApi, call: JmsWebsocket["call"], refresh: () => void, addError: (e: string) => void) {
  let block: ScheduleBlock = {
    id: event.id,
    block_type: event.extendedProps["type"] as any,
    end_time: event.end!.toISOString(),
    start_time: event.start!.toISOString(),
    name: event.extendedProps["name"] as any
  };

  let fut = confirmModal("", {
    title: "Edit Schedule Block",
    data: block,
    okText: "Save",
    cancelText: "Delete",
    okVariant: "success",
    cancelVariant: "danger",
    renderInner: (data, onUpdate) => <ScheduleBlockModalInner data={data} onUpdate={onUpdate} />
  });

  fut
    .then(block => {
      let updates: ScheduleBlockUpdate[] = [];
      for (let key of Object.keys(block)) {
        if (key !== "id") {
          updates.push({ [key]: (block as any)[key] } as any)
        }
      }
      call<"event/schedule_edit">("event/schedule_edit", { block_id: block.id, updates })
        .then(refresh)
        .catch(addError)
    })
    .catch(() => {
      call<"event/schedule_delete">("event/schedule_delete", { block_id: block.id })
        .then(refresh)
        .catch(addError)
    })
}

async function newEventModal(data: DateSelectArg, call: JmsWebsocket["call"], refresh: () => void, addError: (e: string) => void) {
  let block: ScheduleBlock = {
    id: "doesntmatter",
    block_type: { type: "General" },
    name: "My Schedule Block",
    start_time: data.start.toISOString(),
    end_time: data.end.toISOString()
  }

  let fut = confirmModal("", {
    title: "New Schedule Block",
    data: block,
    renderInner: (data, onUpdate) => <ScheduleBlockModalInner data={data} onUpdate={onUpdate} />
  });

  fut.then(block => {
    call<"event/schedule_new_block">("event/schedule_new_block", {
      block_type: block.block_type,
      name: block.name,
      end: block.end_time,
      start: block.start_time
    }).then(refresh).catch(addError)
  })
}

function ScheduleBlockModalInner({ data, onUpdate }: { data: ScheduleBlock, onUpdate: (sb: ScheduleBlock) => void }) {
  return <React.Fragment>
    <InputGroup className="my-1">
      <InputGroup.Text>Name</InputGroup.Text>
      <BufferedFormControl
        auto
        type="text"
        value={data.name}
        onUpdate={v => onUpdate(update(data, { name: { $set: v as string } }))}
      />
    </InputGroup>

    <InputGroup className="my-1">
      <InputGroup.Text>Start Time</InputGroup.Text>
      <BufferedFormControl
        auto
        type="datetime-local"
        value={ moment(data.start_time).format(ELEMENT_FORMAT) }
        onUpdate={v => onUpdate(update(data, { start_time: { $set: moment(v).toISOString() }}))}
      />
    </InputGroup>

    <InputGroup className="my-1">
      <InputGroup.Text>End Time</InputGroup.Text>
      <BufferedFormControl
        auto
        type="datetime-local"
        value={ moment(data.end_time).format(ELEMENT_FORMAT) }
        onUpdate={v => onUpdate(update(data, { end_time: { $set: moment(v).toISOString() }}))}
      />
    </InputGroup>

    <InputGroup className="my-1">
      <InputGroup.Text>Type</InputGroup.Text>
      <Form.Select value={data.block_type.type} onChange={e => onUpdate(update(data, { block_type: { $set: (TYPE_DEFAULTS as any)[e.target.value] } }))}>
        {
          Object.keys(TYPE_MAP).map(t => <option key={t} value={t}>{ (TYPE_MAP as any)[t] }</option>)
        }
      </Form.Select>
    </InputGroup>

    {
      data.block_type.type === "Qualification" && <InputGroup className="mt-3 mb-1">
        <InputGroup.Text>Cycle Time</InputGroup.Text>
        <BufferedFormControl
          auto
          type="number"
          min={3}
          step={0.5}
          value={(data.block_type.cycle_time / 1000 / 60).toFixed(1)}
          onUpdate={v => onUpdate(update(data, { block_type: { cycle_time: { $set: Math.max(3, (v as number)) * 1000 * 60 } } }))}
        />
        <InputGroup.Text>mins</InputGroup.Text>
      </InputGroup>
    }
  </React.Fragment>
}