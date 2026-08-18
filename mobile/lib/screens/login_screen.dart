import 'package:flutter/material.dart';

import '../services/auth_service.dart';
import '../services/server_config.dart';
import '../theme.dart';

/// Login, with the server address as a first-class part of the form.
///
/// The address matters as much as the password here: the same NVR answers on a
/// LAN address at home and on the WireGuard bridge when away.
class LoginScreen extends StatefulWidget {
  const LoginScreen({super.key, required this.servers, required this.auth});

  final ServerConfigService servers;
  final AuthService auth;

  @override
  State<LoginScreen> createState() => _LoginScreenState();
}

class _LoginScreenState extends State<LoginScreen> {
  final _formKey = GlobalKey<FormState>();
  final _username = TextEditingController(text: 'admin');
  final _password = TextEditingController();
  final _host = TextEditingController();
  final _apiPort = TextEditingController();
  final _keycloakPort = TextEditingController();

  bool _useHttps = false;
  bool _showServer = false;
  bool _busy = false;
  bool _testing = false;
  String? _error;
  String? _notice;
  bool _noticeOk = false;

  @override
  void initState() {
    super.initState();
    _loadProfileIntoForm();
  }

  void _loadProfileIntoForm() {
    final profile = widget.servers.current;
    _host.text = profile.host;
    _apiPort.text = '${profile.apiPort}';
    _keycloakPort.text = '${profile.keycloakPort}';
    _useHttps = profile.useHttps;
  }

  @override
  void dispose() {
    _username.dispose();
    _password.dispose();
    _host.dispose();
    _apiPort.dispose();
    _keycloakPort.dispose();
    super.dispose();
  }

  /// The profile as currently typed, without saving it yet.
  ServerProfile _profileFromForm() => widget.servers.current.copyWith(
        host: _host.text.trim(),
        apiPort: int.tryParse(_apiPort.text.trim()) ?? 8888,
        keycloakPort: int.tryParse(_keycloakPort.text.trim()) ?? 8080,
        useHttps: _useHttps,
      );

  Future<void> _saveProfile() =>
      widget.servers.upsert(_profileFromForm(), index: widget.servers.selectedIndex);

  Future<void> _test() async {
    setState(() {
      _testing = true;
      _notice = null;
      _error = null;
    });
    final result = await widget.servers.testConnection(_profileFromForm());
    if (!mounted) return;
    setState(() {
      _testing = false;
      _noticeOk = result.ok;
      _notice = result.message;
    });
  }

  Future<void> _login() async {
    if (!(_formKey.currentState?.validate() ?? false)) return;

    setState(() {
      _busy = true;
      _error = null;
      _notice = null;
    });

    try {
      // Save first, so the login request goes to the server just typed in.
      await _saveProfile();
      await widget.auth.login(_username.text.trim(), _password.text);
      // Navigation is driven by the auth listener in main.dart.
    } on AuthException catch (e) {
      if (!mounted) return;
      setState(() {
        _error = e.message;
        if (e.isNetworkError) _showServer = true;
      });
    } catch (e) {
      if (!mounted) return;
      setState(() => _error = 'Login gagal: $e');
    } finally {
      if (mounted) setState(() => _busy = false);
    }
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      body: SafeArea(
        child: Center(
          child: SingleChildScrollView(
            padding: const EdgeInsets.all(24),
            child: ConstrainedBox(
              constraints: const BoxConstraints(maxWidth: 420),
              child: Form(
                key: _formKey,
                child: Column(
                  crossAxisAlignment: CrossAxisAlignment.stretch,
                  children: [
                    const _Logo(),
                    const SizedBox(height: 28),
                    if (_error != null) _Banner(message: _error!, ok: false),
                    if (_notice != null) _Banner(message: _notice!, ok: _noticeOk),
                    _serverSection(),
                    const SizedBox(height: 16),
                    TextFormField(
                      controller: _username,
                      decoration: const InputDecoration(labelText: 'Username'),
                      textInputAction: TextInputAction.next,
                      autocorrect: false,
                      validator: (v) => (v == null || v.trim().isEmpty) ? 'Username wajib diisi' : null,
                    ),
                    const SizedBox(height: 12),
                    TextFormField(
                      controller: _password,
                      decoration: const InputDecoration(labelText: 'Password'),
                      obscureText: true,
                      textInputAction: TextInputAction.done,
                      onFieldSubmitted: (_) => _login(),
                      validator: (v) => (v == null || v.isEmpty) ? 'Password wajib diisi' : null,
                    ),
                    const SizedBox(height: 20),
                    FilledButton(
                      onPressed: _busy ? null : _login,
                      child: _busy
                          ? const SizedBox(
                              height: 22,
                              width: 22,
                              child: CircularProgressIndicator(strokeWidth: 2, color: Colors.white),
                            )
                          : const Text('Masuk'),
                    ),
                    const SizedBox(height: 20),
                    const Text(
                      'Open-NVR · rekaman tersimpan di server sendiri',
                      textAlign: TextAlign.center,
                      style: TextStyle(color: NvrColors.textSecondary, fontSize: 12),
                    ),
                  ],
                ),
              ),
            ),
          ),
        ),
      ),
    );
  }

  Widget _serverSection() {
    return Container(
      decoration: BoxDecoration(
        color: NvrColors.surface,
        borderRadius: BorderRadius.circular(12),
        border: Border.all(color: NvrColors.border),
      ),
      child: Column(
        children: [
          InkWell(
            onTap: () => setState(() => _showServer = !_showServer),
            borderRadius: BorderRadius.circular(12),
            child: Padding(
              padding: const EdgeInsets.symmetric(horizontal: 14, vertical: 12),
              child: Row(
                children: [
                  const Icon(Icons.dns_outlined, size: 18, color: NvrColors.accent),
                  const SizedBox(width: 10),
                  const Text('Server', style: TextStyle(fontWeight: FontWeight.w600)),
                  const Spacer(),
                  Flexible(
                    child: Text(
                      _host.text.isEmpty ? 'belum diatur' : '${_host.text}:${_apiPort.text}',
                      overflow: TextOverflow.ellipsis,
                      textAlign: TextAlign.end,
                      style: const TextStyle(color: NvrColors.textSecondary, fontSize: 12),
                    ),
                  ),
                  Icon(
                    _showServer ? Icons.expand_less : Icons.expand_more,
                    color: NvrColors.textSecondary,
                  ),
                ],
              ),
            ),
          ),
          if (_showServer) ...[
            const Divider(height: 1),
            Padding(
              padding: const EdgeInsets.all(14),
              child: Column(
                crossAxisAlignment: CrossAxisAlignment.stretch,
                children: [
                  _profileChips(),
                  const SizedBox(height: 14),
                  TextFormField(
                    controller: _host,
                    decoration: const InputDecoration(labelText: 'Host / IP', hintText: '192.168.1.10'),
                    autocorrect: false,
                    keyboardType: TextInputType.url,
                    onChanged: (_) => setState(() {}),
                    validator: (v) => (v == null || v.trim().isEmpty) ? 'Alamat server wajib diisi' : null,
                  ),
                  const SizedBox(height: 10),
                  Row(
                    children: [
                      Expanded(
                        child: TextFormField(
                          controller: _apiPort,
                          decoration: const InputDecoration(labelText: 'Port API'),
                          keyboardType: TextInputType.number,
                          onChanged: (_) => setState(() {}),
                        ),
                      ),
                      const SizedBox(width: 10),
                      Expanded(
                        child: TextFormField(
                          controller: _keycloakPort,
                          decoration: const InputDecoration(labelText: 'Port Keycloak'),
                          keyboardType: TextInputType.number,
                        ),
                      ),
                    ],
                  ),
                  const SizedBox(height: 4),
                  SwitchListTile(
                    value: _useHttps,
                    onChanged: (v) => setState(() => _useHttps = v),
                    title: const Text('HTTPS', style: TextStyle(fontSize: 14)),
                    contentPadding: EdgeInsets.zero,
                    dense: true,
                  ),
                  OutlinedButton.icon(
                    onPressed: _testing ? null : _test,
                    icon: _testing
                        ? const SizedBox(
                            height: 16, width: 16, child: CircularProgressIndicator(strokeWidth: 2))
                        : const Icon(Icons.wifi_tethering, size: 18),
                    label: Text(_testing ? 'Menguji…' : 'Uji koneksi'),
                  ),
                  const SizedBox(height: 8),
                  const Text(
                    'Pakai alamat LAN saat di rumah. Di luar rumah, nyalakan '
                    'WireGuard lalu pilih profil Remote.',
                    style: TextStyle(color: NvrColors.textSecondary, fontSize: 11, height: 1.4),
                  ),
                ],
              ),
            ),
          ],
        ],
      ),
    );
  }

  /// One chip per saved profile — the fast way to flip between home and remote.
  Widget _profileChips() {
    return Wrap(
      spacing: 8,
      runSpacing: 8,
      children: [
        for (var i = 0; i < widget.servers.profiles.length; i++)
          ChoiceChip(
            label: Text(widget.servers.profiles[i].name),
            selected: i == widget.servers.selectedIndex,
            onSelected: (_) async {
              await widget.servers.select(i);
              if (!mounted) return;
              setState(() {
                _loadProfileIntoForm();
                _notice = null;
                _error = null;
              });
            },
            selectedColor: NvrColors.primary,
            backgroundColor: NvrColors.surfaceRaised,
            side: const BorderSide(color: NvrColors.border),
            labelStyle: const TextStyle(fontSize: 12),
          ),
      ],
    );
  }
}

class _Logo extends StatelessWidget {
  const _Logo();

  @override
  Widget build(BuildContext context) {
    return Column(
      children: [
        Container(
          height: 72,
          width: 72,
          decoration: BoxDecoration(
            gradient: const LinearGradient(
              colors: [NvrColors.primary, NvrColors.accent],
              begin: Alignment.topLeft,
              end: Alignment.bottomRight,
            ),
            borderRadius: BorderRadius.circular(20),
          ),
          child: const Icon(Icons.videocam, color: Colors.white, size: 38),
        ),
        const SizedBox(height: 14),
        const Text('Open-NVR', style: TextStyle(fontSize: 26, fontWeight: FontWeight.bold)),
        const Text(
          'Network Video Recorder',
          style: TextStyle(color: NvrColors.textSecondary, fontSize: 13),
        ),
      ],
    );
  }
}

class _Banner extends StatelessWidget {
  const _Banner({required this.message, required this.ok});
  final String message;
  final bool ok;

  @override
  Widget build(BuildContext context) {
    final color = ok ? NvrColors.online : NvrColors.warning;
    return Container(
      margin: const EdgeInsets.only(bottom: 14),
      padding: const EdgeInsets.all(12),
      decoration: BoxDecoration(
        color: color.withValues(alpha: 0.12),
        borderRadius: BorderRadius.circular(10),
        border: Border.all(color: color.withValues(alpha: 0.4)),
      ),
      child: Row(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Icon(ok ? Icons.check_circle_outline : Icons.warning_amber_rounded, color: color, size: 18),
          const SizedBox(width: 10),
          Expanded(
            child: Text(message, style: TextStyle(color: color, fontSize: 13, height: 1.35)),
          ),
        ],
      ),
    );
  }
}
