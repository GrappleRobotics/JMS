import { faCircleNotch, faDownload, faTriangleExclamation } from "@fortawesome/free-solid-svg-icons";
import { FontAwesomeIcon } from "@fortawesome/react-fontawesome";
import React from "react";
import { Button, Col, Row } from "react-bootstrap";
import { Area, CartesianGrid, ComposedChart, Legend, Line, ReferenceArea, Tooltip, XAxis, YAxis } from "recharts";
import { WebsocketComponent } from "support/ws-component";
import { MatchStationStatusRecord, MatchStationStatusRecordKey } from "ws-schema";

const DOWNSAMPLE_STRIDE = 1;

type MatchLogViewState = {
  data?: MatchStationStatusRecord,
  loading: boolean,
  error?: string,
  width: number
}

type MatchLogViewProps = {
  team: number,
  match_id: string,
  autoload?: boolean
};

export default class MatchLogView extends WebsocketComponent<MatchLogViewProps, MatchLogViewState> {
  readonly state: MatchLogViewState = { loading: false, width: 500 };
 
  private ref = React.createRef<HTMLDivElement>();

  componentDidMount = () => {
    this.ref.current!.addEventListener("resize", () => this.recalcSize());
    window.addEventListener("resize", () => this.recalcSize());
    setTimeout(() => this.recalcSize(), 100);

    if (this.props.autoload) {
      this.loadLogs();
    }
  }

  componentDidUpdate = (prevProps: MatchLogViewProps) => {
    if (this.props.match_id != prevProps.match_id || this.props.team != prevProps.team) {
      this.loadLogs();
    }
  }

  loadLogs = () => {
    const key: MatchStationStatusRecordKey = {
      team: this.props.team,
      match_id: this.props.match_id
    };

    this.setState({ loading: true }, () => (
      this.transact<MatchStationStatusRecord | null>({
        Ticket: { Logs: { Load: key } }
      }, "Ticket/Logs/Load")
      .then(data => {
        if (data.msg != null) {
          let record = [];
          
          // The raw data we get is way too much (50ms) - downsample it.
          for (let i = 0; i < data.msg.record.length; i += DOWNSAMPLE_STRIDE) {
            record.push(data.msg.record[i]);
          }

          // localStorage.setItem(JSON.stringify(data.msg.key), JSON.stringify(record));

          this.setState({ loading: false, data: {
            key: data.msg.key,
            record: record
          } });
        } else {
          this.setState({ loading: false, error: "No Record Exists" });
        }
      }).catch((reason) => this.setState({ loading: false, error: reason }))
    ));
  }
  
  renderReferences = (scale: [number, number], axisId?: string) => {
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
      <Area yAxisId={axisId} dataKey={d => d.estop ? scale[1] : undefined} fill="url(#estop)" strokeWidth="0" legendType="none" tooltipType="none" stroke="red" />

    </React.Fragment>
  }

  // ResponsiveContainer doesn't work on some iPads, so we have to do this instead.
  recalcSize = () => {
    if (this.ref.current) {
      let width = this.ref.current.clientWidth;
      this.setState({ width: width - 10 });
    }
  }

  render() {
    const { data, loading, error, width } = this.state;
    return <div ref={this.ref} style={{ width: "100%", maxWidth: "100%" }}>
      {
        error ? (
        <h4 className="text-warning">
          <FontAwesomeIcon icon={faTriangleExclamation} />
          &nbsp; Could not load record: { error }
        </h4>)
        : loading || data == null ? (
          <Button size="lg" onClick={this.loadLogs} disabled={loading}>
            <FontAwesomeIcon icon={loading ? faCircleNotch : faDownload} spin={loading} />
            &nbsp;&nbsp;Load Match Logs
          </Button>
        ) : (
          <React.Fragment>
            <Row>
              <Col>
                <ComposedChart data={data.record} syncId="record" height={250} width={width}>
                  <CartesianGrid />
                  <XAxis label={{value: "Match Time (s)", offset: -5, position: "insideBottom", fill: "#888"}} type="number" dataKey="match_time.secs" tickCount={20} interval="preserveStartEnd" domain={['dataMin', 'dataMax']} />
                  <YAxis label={{value: "Voltage (V)", offset: 20, position: "insideLeft", angle: -90, fill: "#f00"}} yAxisId="battery" type="number" interval="preserveStartEnd" domain={[0, 16]} />
                  <YAxis label={{value: "Time (ms)", offset: 20, position: "insideRight", angle: -90, fill: "#ff8c00"}} yAxisId="rtt" type="number" ticks={[0, 50, 100, 150, 200, 255]} domain={[0, 255]} orientation="right" />

                  { this.renderReferences([0, 16], "battery") }

                  <Tooltip formatter={(v: number) => Math.round(v * 100) / 100} contentStyle={ { backgroundColor: "#202020"} } itemStyle={{ paddingBottom: 0 }} />

                  <Line name="Bat Voltage" yAxisId="battery" dataKey="ds_report.battery" strokeWidth={2} stroke="#f00" dot={false}/>
                  <Line name="Radio RTT" yAxisId="rtt" dataKey="ds_report.rtt" strokeWidth={2} stroke="#ff8c00" dot={false}/>

                </ComposedChart>
              </Col>
            </Row>
            <Row className="mt-3">
              <Col>
                <ComposedChart data={data.record} syncId="record" height={200} width={width}>
                  <XAxis type="number" dataKey="match_time.secs" name="Time" tickCount={20} interval="preserveStartEnd" domain={['dataMin', 'dataMax']} />
                  <YAxis type="number" domain={[0, 1]} ticks={[0, 1]} tickFormatter={t => t ? "True" : "False"} />
                  <YAxis type="number" yAxisId="dummy" tick={false} orientation="right" />

                  { this.renderReferences([0, 1.08]) }

                  <Tooltip formatter={(d: number) => d > 0.5 ? "True" : "False"} contentStyle={ { backgroundColor: "#202020"} } itemStyle={{ paddingBottom: 0 }} />

                  <Line name="Ethernet" dataKey={d => Number(d.ds_eth)}                             strokeOpacity={0.5} strokeWidth={2} stroke="#0f0" dot={false} />
                  <Line name="DS OK"    dataKey={d => Number(d.occupancy === "Occupied") + 0.02}    strokeOpacity={0.5} strokeWidth={2} stroke="#fc00f0" dot={false} />
                  <Line name="Radio"    dataKey={d => Number(d.ds_report?.radio_ping || 0) + 0.04}  strokeOpacity={0.5} strokeWidth={2} stroke="#00e3fc" dot={false} />
                  <Line name="RIO"      dataKey={d => Number(d.ds_report?.rio_ping || 0) + 0.06}    strokeOpacity={0.5} strokeWidth={2} stroke="#8f8f8f" dot={false} />
                  <Line name="Code"     dataKey={d => Number(d.ds_report?.robot_ping || 0) + 0.08}  strokeOpacity={0.5} strokeWidth={2} stroke="#ffd000" dot={false} />

                  <Legend />
                </ComposedChart>
              </Col>
            </Row>
          </React.Fragment> 
        )
      }
    </div>
  }
}