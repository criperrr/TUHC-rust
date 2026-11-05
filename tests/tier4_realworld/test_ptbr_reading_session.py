"""
Tier 4: Real-World Scenarios - Full Homestuck PT-BR Localized Reading Session
Simulates enabling the PT-BR community mod, reading translated pages, verifying pesterlog hooks, and loading localized media.
"""

from tests.common import TuhcTestCase, evaluate_ptbr_log_button_text


class TestPtBrReadingSessionScenario(TuhcTestCase):

    def test_complete_ptbr_reading_session(self):
        # 1. Enable PT-BR mod
        toggle_resp = self.client.ipc("toggle_mod", {"mod_id": "Homestuck PT-BR", "enabled": True})
        self.assertStatus(toggle_resp, 200)

        # 2. Read translated Act 1 opener (001901)
        p1 = self.client.ipc("get_story_page", {"page_id": "001901"}).json()
        self.assertIn("Um jovem rapaz está em seu quarto", p1["content"])
        self.assertIn("13 de Abril de 2009", p1["content"])

        # 3. Read translated page 001902 and verify localized media
        p2 = self.client.ipc("get_story_page", {"page_id": "001902"}).json()
        self.assertEqual(p2["title"], "Insira o nome.")

        # Localized 00002.gif asset served via VFS
        img_resp = self.client.get("/assets/storyfiles/hs2/00002.gif")
        self.assertStatus(img_resp, 200)
        self.assertEqual(len(img_resp.data), 191716)

        # 4. Read pesterlog page 002060
        p2060 = self.client.ipc("get_story_page", {"page_id": "002060"}).json()
        self.assertEqual(p2060["title"], "John: Vá para a sacada.")
        self.assertIn("EB: ei, eu tô na sacada agora.", p2060["content"])

        # 5. Verify vueHooks dynamic logButtonText evaluation
        btn_closed = evaluate_ptbr_log_button_text(p2060["content"], log_hidden=True)
        self.assertEqual(btn_closed, "Abrir pesterlog")

        btn_opened = evaluate_ptbr_log_button_text(p2060["content"], log_hidden=False)
        self.assertEqual(btn_opened, "Fechar pesterlog")

        # 6. Clean up: disable mod
        self.client.ipc("toggle_mod", {"mod_id": "Homestuck PT-BR", "enabled": False})
