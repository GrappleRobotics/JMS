// Configuration from JMS
$stations['blue1']['team'] = {{ blue1.0 }};
$stations['blue2']['team'] = {{ blue2.0 }};
$stations['blue3']['team'] = {{ blue3.0 }};
$stations['red1']['team'] = {{ red1.0 }};
$stations['red2']['team'] = {{ red2.0 }};
$stations['red3']['team'] = {{ red3.0 }};

// PfSense Script (doesn't change)
foreach($config['vlans']['vlan'] as $k => $value) {
  $vlans[$value['tag']] = $value['vlanif'];
}

$stations['blue1']['vlanif'] = $vlans[10];
$stations['blue2']['vlanif'] = $vlans[20];
$stations['blue3']['vlanif'] = $vlans[30];
$stations['red1']['vlanif'] = $vlans[40];
$stations['red2']['vlanif'] = $vlans[50];
$stations['red3']['vlanif'] = $vlans[60];

$rule['id'] = 'ds2fms_tcp';
$rule['protocol'] = 'tcp';
$rule['destination'] = '10.0.100.5';
$rule['port'] = '1750';
$firewall_rules[] = $rule;
unset($rule);

$rule['id'] = 'ds2fms_udp';
$rule['protocol'] = 'udp';
$rule['destination'] = '10.0.100.5';
$rule['port'] = '1160';
$firewall_rules[] = $rule;
unset($rule);

$rule['id'] = 'ds2fms_icmp';
$rule['protocol'] = 'icmp';
$rule['destination'] = '10.0.100.5';
$firewall_rules[] = $rule;
unset($rule);

$tracker = time();

// Create interfaces
foreach($stations as $k => $value) {
  if ($value['team'] > 0) {
    $team_high = floor($value['team'] / 100);
    $team_low = $value['team'] % 100;

    $config['interfaces'][$k]['enable'] = true;
    $config['interfaces'][$k]['if'] = $value['vlanif'];
    $config['interfaces'][$k]['desc'] = $k;
    $config['interfaces'][$k]['ipaddr'] = "10." . $team_high . "." . $team_low . ".4";
    $config['interfaces'][$k]['subnet'] = 24;
    
    // FRC Radios are typically in the range .200 to .220
    $config['dhcpd'][$k]['enable'] = true;
    $config['dhcpd'][$k]['range']['from'] = "10." . $team_high . "." . $team_low . ".100";
    $config['dhcpd'][$k]['range']['to'] = "10." . $team_high . "." . $team_low . ".150";

    // Configure rules
    foreach($firewall_rules as $k2 => $rule_template) {
      $rule['id'] = $k . $rule_template['id'];
      $rule['type'] = 'pass';
      $rule['ipprotocol'] = 'inet';
      $rule['protocol'] = $rule_template['protocol'];
      $rule['descr'] = 'JMS AUTO ' . $rule_template['id'];
      $rule['interface'] = $k;
      $rule['tracker'] = $tracker;
      $tracker += 1;
      $rule['source']['address'] = "10." . $team_high . "." . $team_low . ".0/24";    // For some reason we can't assign the network itself to the firewall rule. 
      $rule['destination']['address'] = $rule_template['destination'];
      $rule['destination']['port'] = $rule_template['port'];

      $rule_idx = array_search($rule['id'], array_column($config['filter']['rule'], 'id'));
      if (empty($rule_idx)) {
        $config['filter']['rule'][] = $rule;
      } else {
        $config['filter']['rule'][$rule_idx] = $rule;
      }
    }
  } else {
    $config['interfaces'][$k]['enable'] = false;
    $config['dhcpd'][$k]['enable'] = false;
  }
  parse_config(true);
  write_config();
  filter_configure();
  
  interface_bring_down($k, false, $config['interfaces'][$k]);
  interface_configure($k, true);
}
services_dhcpd_configure();
exec