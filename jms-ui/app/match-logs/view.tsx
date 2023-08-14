import React, { useEffect, useRef, useState } from "react";
import { MatchLog } from "../ws-schema";
import { Col, Row } from "react-bootstrap";
import { Area, CartesianGrid, ComposedChart, Legend, Line, ReferenceArea, Tooltip, XAxis, YAxis } from "recharts";

export default function MatchLogView({ matchLog }: { matchLog: MatchLog }) {
  const ref = useRef<HTMLDivElement>(null);

  const [ width, setWidth ] = useState<number>(500);

  const recalcSize = () => {
    if (ref.current) {
      let width = ref.current.clientWidth;
      setWidth(width - 10);
    }
  };

  useEffect(() => {
    ref.current!.addEventListener("resize", () => recalcSize());
    window.addEventListener("resize", () => recalcSize());
    setTimeout(() => recalcSize(), 100);
  }, [])

  const renderReferences = (scale: [number, number], axisId?: string) => {
    return <React.Fragment>
      <defs>
        <linearGradient id="estop" x1={0} x2="0.2" spreadMethod="repeat" gradientTransform="rotate(45)">
          <stop offset="0px" stop-color="#730000" />
          <stop offset="50%" stop-color="#730000" />
          <stop offset="51%" stop-color="#400000" />
          <stop offset="100%" stop-color="#400000" />
        </linearGradient>
      </defs>

      <ReferenceArea yAxisId={axisId} x1={0} x2={15} y1={scale[0]} y2={scale[1]} fill="purple" fillOpacity={0.15} />
      <ReferenceArea yAxisId={axisId} x1={16} x2={135} y1={scale[0]} y2={scale[1]} fill="orange" fillOpacity={0.15} />
      <Area yAxisId={axisId} dataKey={d => d.report?.estop ? scale[1] : undefined} fill="url(#estop)" strokeWidth="0" legendType="none" tooltipType="none" stroke="red" />

    </React.Fragment>
  }

  return <div ref={ref} style={{ width: "100%", maxWidth: "100%" }}>
    <Row>
      <Col>
        <ComposedChart data={matchLog.timeseries} syncId="record" height={250} width={width}>
          <CartesianGrid />
          <XAxis label={{value: "Match Time (s)", offset: -5, position: "insideBottom", fill: "#888"}} type="number" dataKey={d => d.time / 1000} tickCount={20} interval="preserveStartEnd" domain={['dataMin', 'dataMax']} />
          <YAxis label={{value: "Voltage (V)", offset: 20, position: "insideLeft", angle: -90, fill: "#f00"}} yAxisId="battery" type="number" interval="preserveStartEnd" domain={[0, 16]} />
          <YAxis label={{value: "Time (ms)", offset: 20, position: "insideRight", angle: -90, fill: "#ff8c00"}} yAxisId="rtt" type="number" ticks={[0, 50, 100, 150, 200, 255]} domain={[0, 255]} orientation="right" />

          { renderReferences([0, 16], "battery") }

          <Tooltip formatter={(v: number) => Math.round(v * 100) / 100} contentStyle={ { backgroundColor: "#202020"} } itemStyle={{ paddingBottom: 0 }} />

          <Line name="Bat Voltage" yAxisId="battery" dataKey={d => d.report?.battery_voltage || 0} strokeWidth={2} stroke="#f00" dot={false}/>
          <Line name="Radio RTT" yAxisId="rtt" dataKey={d => d.report?.rtt} strokeWidth={2} stroke="#ff8c00" dot={false}/>
        </ComposedChart>
      </Col>
    </Row>
    <Row className="mt-3">
      <Col>
        <ComposedChart data={matchLog.timeseries} syncId="record" height={200} width={width}>
          <XAxis type="number" dataKey={d => d.time / 1000} name="Time" tickCount={20} interval="preserveStartEnd" domain={['dataMin', 'dataMax']} />
          <YAxis type="number" domain={[0, 1]} ticks={[0, 1]} tickFormatter={t => t ? "True" : "False"} />
          <YAxis type="number" yAxisId="dummy" tick={false} orientation="right" />

          { renderReferences([0, 1.08]) }

          <Tooltip formatter={(d: number) => d > 0.5 ? "True" : "False"} contentStyle={ { backgroundColor: "#202020"} } itemStyle={{ paddingBottom: 0 }} />

          <Line name="DS OK"    dataKey={d => Number(d.report ? 1 : 0) + 0.02}  strokeOpacity={0.5} strokeWidth={2} stroke="#00e300" dot={false} />
          <Line name="Radio"    dataKey={d => Number(d.report?.radio_ping || 0) + 0.04}  strokeOpacity={0.5} strokeWidth={2} stroke="#00e3fc" dot={false} />
          <Line name="RIO"      dataKey={d => Number(d.report?.rio_ping || 0) + 0.06}    strokeOpacity={0.5} strokeWidth={2} stroke="#8f8f8f" dot={false} />
          <Line name="Code"     dataKey={d => Number(d.report?.robot_ping || 0) + 0.08}  strokeOpacity={0.5} strokeWidth={2} stroke="#ffd000" dot={false} />

          <Legend />
        </ComposedChart>
      </Col>
    </Row>
  </div>
}